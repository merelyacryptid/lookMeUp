import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Definition {
  word: string;
  part_of_speech: string;
  ipa: string;
  meanings: string[];
  lookup_count: number;
  is_saved: boolean;
}

interface WordHistoryItem {
  word: string;
  part_of_speech: string;
  ipa: string;
  meanings: string[];
  lookup_count: number;
  is_saved: boolean;
  last_looked_up: number;
}

type LookupPayload = { Ok: Definition } | { Err: string };

// App State
let activeWord = "";
let isExpanded = false;
let wordbankFilter: "all" | "saved" = "all";
let wordHistoryItems: WordHistoryItem[] = [];

// DOM Elements - Compact Popup
const popupView = document.querySelector<HTMLElement>("#popup-view")!;
const companionSearch = document.querySelector<HTMLInputElement>("#companion-search")!;
const btnOpenWordbank = document.querySelector<HTMLButtonElement>("#btn-open-wordbank")!;
const btnClosePopup = document.querySelector<HTMLButtonElement>("#btn-close-popup")!;
const popupWord = document.querySelector<HTMLElement>("#popup-word")!;
const popupIpa = document.querySelector<HTMLElement>("#popup-ipa")!;
const popupSpeakBtn = document.querySelector<HTMLButtonElement>("#popup-speak-btn")!;
const popupPos = document.querySelector<HTMLElement>("#popup-pos")!;
const popupCount = document.querySelector<HTMLElement>("#popup-count")!;
const popupSaveBtn = document.querySelector<HTMLButtonElement>("#popup-save-btn")!;
const popupSaveText = document.querySelector<HTMLElement>("#popup-save-text")!;
const popupBody = document.querySelector<HTMLElement>("#popup-body")!;
const hintHotkey = document.querySelector<HTMLElement>("#hint-hotkey")!;
const footerHotkeyPill = document.querySelector<HTMLElement>("#footer-hotkey-pill")!;

// DOM Elements - Expanded Word Bank & Settings
const wordbankView = document.querySelector<HTMLElement>("#wordbank-view")!;
const btnBackToPopup = document.querySelector<HTMLButtonElement>("#btn-back-to-popup")!;
const btnCloseWordbank = document.querySelector<HTMLButtonElement>("#btn-close-wordbank")!;
const tabAll = document.querySelector<HTMLButtonElement>("#tab-all")!;
const tabSaved = document.querySelector<HTMLButtonElement>("#tab-saved")!;
const tabSettings = document.querySelector<HTMLButtonElement>("#tab-settings")!;
const paneVocabulary = document.querySelector<HTMLElement>("#pane-vocabulary")!;
const paneSettings = document.querySelector<HTMLElement>("#pane-settings")!;
const statAllCount = document.querySelector<HTMLElement>("#stat-all-count")!;
const statSavedCount = document.querySelector<HTMLElement>("#stat-saved-count")!;
const wordbankFilterInput = document.querySelector<HTMLInputElement>("#wordbank-filter-input")!;
const wordbankItemsList = document.querySelector<HTMLElement>("#wordbank-items-list")!;

// Settings Elements
const activeHotkeyDisplay = document.querySelector<HTMLElement>("#active-hotkey-display")!;
const presetButtons = document.querySelectorAll<HTMLButtonElement>(".preset-btn");
const customHotkeyInput = document.querySelector<HTMLInputElement>("#custom-hotkey-input")!;
const btnApplyHotkey = document.querySelector<HTMLButtonElement>("#btn-apply-hotkey")!;
const hotkeyStatusMsg = document.querySelector<HTMLElement>("#hotkey-status-msg")!;

// ----------------------------------------------------
// 1. Definition Display
// ----------------------------------------------------
function showDefinition(def: Definition) {
  activeWord = def.word;
  companionSearch.value = def.word;

  popupWord.textContent = def.word;
  popupIpa.textContent = def.ipa || "";
  popupSpeakBtn.classList.remove("hidden");
  popupPos.textContent = def.part_of_speech || "word";
  popupCount.textContent = def.lookup_count > 1 ? `${def.lookup_count}x` : "";

  updateSaveBtnState(def.is_saved);

  if (def.meanings && def.meanings.length > 0) {
    popupBody.innerHTML = `
      <ol>
        ${def.meanings.slice(0, 5).map((m) => `<li>${escapeHtml(m)}</li>`).join("")}
      </ol>
    `;
  } else {
    popupBody.innerHTML = `<p class="def-hint">No definitions available for this entry.</p>`;
  }
}

function showError(msg: string) {
  activeWord = "";
  popupWord.textContent = "Not Found";
  popupIpa.textContent = "";
  popupSpeakBtn.classList.add("hidden");
  popupPos.textContent = "";
  popupCount.textContent = "";
  updateSaveBtnState(false);
  popupBody.innerHTML = `<p class="def-hint" style="color:#f87171;">${escapeHtml(msg)}</p>`;
}

async function speakCurrentWord(word: string) {
  if (!word) return;
  popupSpeakBtn.classList.add("speaking");

  const hasWebSpeech = "speechSynthesis" in window && window.speechSynthesis.getVoices().length > 0;
  if (hasWebSpeech) {
    window.speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(word);
    utterance.lang = "en-US";
    utterance.onend = () => popupSpeakBtn.classList.remove("speaking");
    utterance.onerror = () => {
      popupSpeakBtn.classList.remove("speaking");
      invoke("speak_word", { word }).catch(() => {});
    };
    window.speechSynthesis.speak(utterance);
  } else {
    try {
      await invoke("speak_word", { word });
    } catch (err) {
      console.error("Offline speech failed:", err);
    } finally {
      setTimeout(() => popupSpeakBtn.classList.remove("speaking"), 600);
    }
  }
}

popupSpeakBtn.addEventListener("click", () => {
  if (activeWord) speakCurrentWord(activeWord);
});

function updateSaveBtnState(isSaved: boolean) {
  if (isSaved) {
    popupSaveBtn.classList.add("saved");
    popupSaveText.textContent = "Saved";
  } else {
    popupSaveBtn.classList.remove("saved");
    popupSaveText.textContent = "Save Word";
  }
}

async function searchWord(query: string) {
  const clean = query.trim();
  if (!clean) return;

  popupWord.textContent = clean;
  popupPos.textContent = "...";
  popupCount.textContent = "";
  popupBody.innerHTML = `<p class="def-hint">Searching offline dictionary...</p>`;

  try {
    const def = await invoke<Definition>("lookup_word", { word: clean });
    showDefinition(def);
  } catch (err) {
    showError(String(err));
  }
}

// ----------------------------------------------------
// 2. View Switching (Compact vs Expanded)
// ----------------------------------------------------
async function switchView(expanded: boolean) {
  isExpanded = expanded;
  if (expanded) {
    popupView.classList.add("hidden");
    wordbankView.classList.remove("hidden");
    await invoke("resize_window", { expanded: true });
    await loadWordBank();
  } else {
    wordbankView.classList.add("hidden");
    popupView.classList.remove("hidden");
    await invoke("resize_window", { expanded: false });
  }
}

btnOpenWordbank.addEventListener("click", () => switchView(true));
btnBackToPopup.addEventListener("click", () => switchView(false));
btnClosePopup.addEventListener("click", () => invoke("hide_window"));
btnCloseWordbank.addEventListener("click", () => invoke("hide_window"));

// ----------------------------------------------------
// 3. Hotkey Listeners & Event Subscribers
// ----------------------------------------------------
listen<LookupPayload>("show-definition", (event) => {
  switchView(false);
  const payload = event.payload;
  if ("Ok" in payload) {
    showDefinition(payload.Ok);
  } else {
    showError(payload.Err);
  }
});

listen("open-wordbank", () => {
  switchView(true);
});

listen("open-search", () => {
  switchView(false);
  companionSearch.focus();
  companionSearch.select();
});

// Auto-dismiss on blur when in compact mode
window.addEventListener("blur", () => {
  if (!isExpanded) {
    invoke("hide_window");
  }
});

// Escape key to dismiss
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    invoke("hide_window");
  }
});

// Search input submit
companionSearch.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    searchWord(companionSearch.value);
  }
});

// Save word toggle button
popupSaveBtn.addEventListener("click", async () => {
  if (!activeWord) return;
  try {
    const isSaved = await invoke<boolean>("toggle_save_word", { word: activeWord });
    updateSaveBtnState(isSaved);
    // Sync item in word bank list if loaded
    const item = wordHistoryItems.find((i) => i.word.toLowerCase() === activeWord.toLowerCase());
    if (item) item.is_saved = isSaved;
  } catch (err) {
    console.error("Failed to toggle save word:", err);
  }
});

// ----------------------------------------------------
// 4. Word Bank & Vocabulary Management
// ----------------------------------------------------
async function loadWordBank() {
  try {
    wordHistoryItems = await invoke<WordHistoryItem[]>("get_word_history");
    updateWordBankStats();
    renderWordBank();
  } catch (err) {
    console.error("Failed to load word history:", err);
    wordbankItemsList.innerHTML = `<div class="empty-state">Failed to load vocabulary history.</div>`;
  }
}

function updateWordBankStats() {
  const total = wordHistoryItems.length;
  const saved = wordHistoryItems.filter((i) => i.is_saved).length;
  statAllCount.textContent = String(total);
  statSavedCount.textContent = String(saved);
}

function renderWordBank() {
  const query = wordbankFilterInput.value.toLowerCase().trim();
  let filtered = wordHistoryItems;

  if (wordbankFilter === "saved") {
    filtered = filtered.filter((i) => i.is_saved);
  }

  if (query) {
    filtered = filtered.filter(
      (i) =>
        i.word.toLowerCase().includes(query) ||
        i.meanings.some((m) => m.toLowerCase().includes(query))
    );
  }

  if (filtered.length === 0) {
    const msg =
      wordbankFilter === "saved"
        ? `No saved words yet.<br><small style="color:#52525b;margin-top:4px;display:block;">Click "Save Word" on any definition card to pin vocabulary here.</small>`
        : `No lookup history recorded yet.<br><small style="color:#52525b;margin-top:4px;display:block;">Press your hotkey over any word in any desktop app to start building your personal memory bank.</small>`;
    wordbankItemsList.innerHTML = `<div class="empty-state">${msg}</div>`;
    return;
  }

  wordbankItemsList.innerHTML = "";
  filtered.forEach((item) => {
    const card = document.createElement("div");
    card.className = "wordbank-card";

    const meaningExcerpt = item.meanings && item.meanings.length > 0 ? item.meanings[0] : "";

    card.innerHTML = `
      <div class="wordbank-info">
        <div class="wordbank-top">
          <span class="wordbank-word">${escapeHtml(item.word)}</span>
          ${item.ipa ? `<span class="wordbank-ipa">${escapeHtml(item.ipa)}</span>` : ""}
          <span class="wordbank-pos">${escapeHtml(item.part_of_speech || "word")}</span>
          <span class="wordbank-badge">${item.lookup_count}x</span>
        </div>
        <div class="wordbank-meaning">${escapeHtml(meaningExcerpt)}</div>
      </div>
      <div class="wordbank-actions">
        <button class="star-btn ${item.is_saved ? "saved" : ""}" title="${item.is_saved ? "Unsave" : "Save"}">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="${item.is_saved ? "currentColor" : "none"}" stroke="currentColor" stroke-width="2">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
          </svg>
        </button>
        <button class="del-btn" title="Delete from history">&times;</button>
      </div>
    `;

    // Click card to open in compact definition view
    card.querySelector(".wordbank-info")?.addEventListener("click", () => {
      showDefinition({
        word: item.word,
        part_of_speech: item.part_of_speech,
        ipa: item.ipa,
        meanings: item.meanings,
        lookup_count: item.lookup_count,
        is_saved: item.is_saved,
      });
      switchView(false);
    });

    // Star/Save toggle
    card.querySelector(".star-btn")?.addEventListener("click", async (e) => {
      e.stopPropagation();
      try {
        const newSaved = await invoke<boolean>("toggle_save_word", { word: item.word });
        item.is_saved = newSaved;
        updateWordBankStats();
        renderWordBank();
      } catch (err) {
        console.error("Failed to toggle word save:", err);
      }
    });

    // Delete word
    card.querySelector(".del-btn")?.addEventListener("click", async (e) => {
      e.stopPropagation();
      try {
        await invoke("delete_history_word", { word: item.word });
        wordHistoryItems = wordHistoryItems.filter((i) => i.word.toLowerCase() !== item.word.toLowerCase());
        updateWordBankStats();
        renderWordBank();
      } catch (err) {
        console.error("Failed to delete word:", err);
      }
    });

    wordbankItemsList.appendChild(card);
  });
}

wordbankFilterInput.addEventListener("input", renderWordBank);

// Tabs Navigation
tabAll.addEventListener("click", () => {
  wordbankFilter = "all";
  tabAll.classList.add("active");
  tabSaved.classList.remove("active");
  tabSettings.classList.remove("active");
  paneVocabulary.classList.remove("hidden");
  paneSettings.classList.add("hidden");
  renderWordBank();
});

tabSaved.addEventListener("click", () => {
  wordbankFilter = "saved";
  tabSaved.classList.add("active");
  tabAll.classList.remove("active");
  tabSettings.classList.remove("active");
  paneVocabulary.classList.remove("hidden");
  paneSettings.classList.add("hidden");
  renderWordBank();
});

tabSettings.addEventListener("click", () => {
  tabSettings.classList.add("active");
  tabAll.classList.remove("active");
  tabSaved.classList.remove("active");
  paneVocabulary.classList.add("hidden");
  paneSettings.classList.remove("hidden");
  loadHotkeySettings();
});

// ----------------------------------------------------
// 5. Settings: Hotkey Configuration
// ----------------------------------------------------
async function loadHotkeySettings() {
  try {
    const key = await invoke<string>("get_current_hotkey");
    activeHotkeyDisplay.textContent = key.toUpperCase();
    hintHotkey.textContent = key.toUpperCase();
    footerHotkeyPill.textContent = key.toUpperCase();
    customHotkeyInput.value = key;
  } catch (err) {
    console.error("Failed to fetch current hotkey:", err);
  }
}

async function applyNewHotkey(shortcut: string) {
  hotkeyStatusMsg.textContent = "Updating shortcut...";
  hotkeyStatusMsg.className = "status-msg";

  try {
    const saved = await invoke<string>("set_current_hotkey", { newHotkey: shortcut });
    activeHotkeyDisplay.textContent = saved.toUpperCase();
    hintHotkey.textContent = saved.toUpperCase();
    footerHotkeyPill.textContent = saved.toUpperCase();
    customHotkeyInput.value = saved;
    hotkeyStatusMsg.textContent = `Shortcut updated to ${saved.toUpperCase()}!`;
    hotkeyStatusMsg.className = "status-msg success";
  } catch (err) {
    hotkeyStatusMsg.textContent = `Failed: ${err}`;
    hotkeyStatusMsg.className = "status-msg error";
  }
}

presetButtons.forEach((btn) => {
  btn.addEventListener("click", () => {
    const key = btn.dataset.key;
    if (key) applyNewHotkey(key);
  });
});

btnApplyHotkey.addEventListener("click", () => {
  const val = customHotkeyInput.value.trim();
  if (val) applyNewHotkey(val);
});

function escapeHtml(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

// Initial Boot
loadHotkeySettings();
