import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

interface Definition {
  word: string;
  part_of_speech: string;
  meanings: string[];
}

interface EpubChapter {
  title: string;
  content: string;
}

interface EpubBook {
  title: string;
  cover_data: string | null;
  chapters: EpubChapter[];
}

// State
let currentBook: EpubBook | null = null;
let currentChapterIdx = 0;

// DOM Elements
const readerScrollContainer = document.querySelector<HTMLElement>("#reader-scroll-container")!;
const readerContent = document.querySelector<HTMLElement>("#reader-content")!;
const openBtn = document.querySelector<HTMLElement>("#open-btn");
const tocBtn = document.querySelector<HTMLElement>("#toc-btn");
const typoBtn = document.querySelector<HTMLElement>("#typo-btn");
const prevBtn = document.querySelector<HTMLButtonElement>("#prev-btn");
const nextBtn = document.querySelector<HTMLButtonElement>("#next-btn");
const chapterLabel = document.querySelector<HTMLElement>("#chapter-label");
const bookTitleFooter = document.querySelector<HTMLElement>("#book-title-footer");
const progressBar = document.querySelector<HTMLElement>("#progress-bar");
const progressText = document.querySelector<HTMLElement>("#progress-text");

// Popups & Drawers
const typoPopover = document.querySelector<HTMLElement>("#typo-popover")!;
const chapterDrawer = document.querySelector<HTMLElement>("#chapter-drawer")!;
const drawerClose = document.querySelector<HTMLElement>("#drawer-close")!;
const drawerList = document.querySelector<HTMLElement>("#drawer-list")!;
const popupCard = document.querySelector<HTMLElement>("#popup-card")!;
const popupWord = document.querySelector<HTMLElement>("#popup-word")!;
const popupPos = document.querySelector<HTMLElement>("#popup-pos")!;
const popupBody = document.querySelector<HTMLElement>("#popup-body")!;
const popupClose = document.querySelector<HTMLElement>("#popup-close")!;

// ----------------------------------------------------
// 1. Chapter Rendering & Pagination
// ----------------------------------------------------
function renderChapter(idx: number) {
  if (!currentBook || !currentBook.chapters[idx]) return;
  currentChapterIdx = idx;

  const ch = currentBook.chapters[idx];
  readerContent.innerHTML = ch.content;

  if (chapterLabel) {
    chapterLabel.textContent = `${idx + 1} / ${currentBook.chapters.length}`;
  }
  if (prevBtn) prevBtn.disabled = idx <= 0;
  if (nextBtn) nextBtn.disabled = idx >= currentBook.chapters.length - 1;

  readerScrollContainer.scrollTo({ top: 0, behavior: "smooth" });
  updateProgress();
  highlightActiveDrawerItem();
}

function updateProgress() {
  if (!currentBook) return;
  const total = currentBook.chapters.length;
  const pct = Math.round(((currentChapterIdx + 1) / total) * 100);
  if (progressBar) progressBar.style.width = `${pct}%`;
  if (progressText) progressText.textContent = `${pct}% (Ch. ${currentChapterIdx + 1}/${total})`;
}

prevBtn?.addEventListener("click", () => {
  if (currentBook && currentChapterIdx > 0) renderChapter(currentChapterIdx - 1);
});

nextBtn?.addEventListener("click", () => {
  if (currentBook && currentChapterIdx < currentBook.chapters.length - 1) {
    renderChapter(currentChapterIdx + 1);
  }
});

// ----------------------------------------------------
// 2. Open EPUB via Native Dialog
// ----------------------------------------------------
openBtn?.addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "EPUB Documents", extensions: ["epub"] }],
    });

    if (selected) {
      const pathStr = Array.isArray(selected) ? selected[0] : selected;
      if (bookTitleFooter) bookTitleFooter.textContent = "Loading book...";

      const book = await invoke<EpubBook>("load_epub", { path: pathStr });
      currentBook = book;
      if (bookTitleFooter) bookTitleFooter.textContent = book.title;

      populateChapterDrawer();
      renderChapter(0);
    }
  } catch (err) {
    console.error("Open dialog error:", err);
    alert(`Could not open or render EPUB: ${err}`);
  }
});

// ----------------------------------------------------
// 3. Chapter Drawer (Table of Contents)
// ----------------------------------------------------
function populateChapterDrawer() {
  if (!currentBook) return;
  drawerList.innerHTML = "";

  currentBook.chapters.forEach((ch, idx) => {
    const item = document.createElement("div");
    item.className = "drawer-item";
    item.textContent = ch.title;
    item.addEventListener("click", () => {
      renderChapter(idx);
      chapterDrawer.classList.add("hidden");
    });
    drawerList.appendChild(item);
  });
}

function highlightActiveDrawerItem() {
  const items = drawerList.querySelectorAll(".drawer-item");
  items.forEach((item, idx) => {
    if (idx === currentChapterIdx) {
      item.classList.add("active");
    } else {
      item.classList.remove("active");
    }
  });
}

tocBtn?.addEventListener("click", () => {
  chapterDrawer.classList.toggle("hidden");
  typoPopover.classList.add("hidden");
});

drawerClose?.addEventListener("click", () => {
  chapterDrawer.classList.add("hidden");
});

// ----------------------------------------------------
// 4. Typography Customizer (Matching Screenshot)
// ----------------------------------------------------
const sliderFontSize = document.querySelector<HTMLInputElement>("#slider-font-size")!;
const selectFontFamily = document.querySelector<HTMLSelectElement>("#select-font-family")!;
const sliderLineHeight = document.querySelector<HTMLInputElement>("#slider-line-height")!;
const sliderWordSpace = document.querySelector<HTMLInputElement>("#slider-word-space")!;
const sliderParaSpace = document.querySelector<HTMLInputElement>("#slider-para-space")!;
const alignButtons = document.querySelectorAll<HTMLButtonElement>(".align-btn");
const resetTypoBtn = document.querySelector<HTMLButtonElement>("#reset-typo-btn")!;

const DEFAULT_TYPO = {
  fontSize: "19",
  fontFamily: "Constantia, serif",
  lineHeight: "1.8",
  wordSpace: "0",
  paraSpace: "20",
  textAlign: "left",
};

function loadTypoSettings() {
  const saved = localStorage.getItem("lookmeup_typography");
  const settings = saved ? JSON.parse(saved) : DEFAULT_TYPO;
  applyTypo(settings);
}

function applyTypo(s: typeof DEFAULT_TYPO) {
  document.documentElement.style.setProperty("--reader-font-size", `${s.fontSize}px`);
  document.documentElement.style.setProperty("--reader-font-family", s.fontFamily);
  document.documentElement.style.setProperty("--reader-line-height", s.lineHeight);
  document.documentElement.style.setProperty("--reader-word-spacing", `${s.wordSpace}px`);
  document.documentElement.style.setProperty("--reader-para-spacing", `${s.paraSpace}px`);
  document.documentElement.style.setProperty("--reader-text-align", s.textAlign);

  sliderFontSize.value = s.fontSize;
  selectFontFamily.value = s.fontFamily;
  sliderLineHeight.value = s.lineHeight;
  sliderWordSpace.value = s.wordSpace;
  sliderParaSpace.value = s.paraSpace;

  alignButtons.forEach((btn) => {
    btn.classList.toggle("active", btn.dataset.align === s.textAlign);
  });

  localStorage.setItem("lookmeup_typography", JSON.stringify(s));
}

function saveCurrentTypo() {
  const activeAlign = document.querySelector<HTMLButtonElement>(".align-btn.active")?.dataset.align || "left";
  const s = {
    fontSize: sliderFontSize.value,
    fontFamily: selectFontFamily.value,
    lineHeight: sliderLineHeight.value,
    wordSpace: sliderWordSpace.value,
    paraSpace: sliderParaSpace.value,
    textAlign: activeAlign,
  };
  applyTypo(s);
}

sliderFontSize.addEventListener("input", saveCurrentTypo);
selectFontFamily.addEventListener("change", saveCurrentTypo);
sliderLineHeight.addEventListener("input", saveCurrentTypo);
sliderWordSpace.addEventListener("input", saveCurrentTypo);
sliderParaSpace.addEventListener("input", saveCurrentTypo);

alignButtons.forEach((btn) => {
  btn.addEventListener("click", () => {
    alignButtons.forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    saveCurrentTypo();
  });
});

resetTypoBtn.addEventListener("click", () => {
  applyTypo(DEFAULT_TYPO);
});

typoBtn?.addEventListener("click", () => {
  typoPopover.classList.toggle("hidden");
  chapterDrawer.classList.add("hidden");
});

// ----------------------------------------------------
// 5. Touch & Mouse Word Lookup
// ----------------------------------------------------
function hidePopup() {
  popupCard.classList.add("hidden");
}

popupClose.addEventListener("click", hidePopup);

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    hidePopup();
    typoPopover.classList.add("hidden");
    chapterDrawer.classList.add("hidden");
  }
});

document.addEventListener("pointerdown", (e) => {
  const target = e.target as Node;
  if (!popupCard.contains(target)) hidePopup();
  if (!typoPopover.contains(target) && !typoBtn?.contains(target)) {
    typoPopover.classList.add("hidden");
  }
});

async function handleSelection() {
  const selection = window.getSelection();
  if (!selection || selection.isCollapsed) return;

  const rawText = selection.toString().trim();
  const cleanWord = rawText.replace(/^[^\w]+|[^\w]+$/g, "");
  if (!cleanWord || cleanWord.length > 40) return;

  const range = selection.getRangeAt(0);
  const rect = range.getBoundingClientRect();

  const popupWidth = 320;
  const popupHeight = 180;

  let left = rect.left + rect.width / 2 - popupWidth / 2;
  left = Math.max(12, Math.min(left, window.innerWidth - popupWidth - 12));

  let top = rect.top - popupHeight - 12;
  if (top < 60) top = rect.bottom + 12;

  popupCard.style.left = `${left}px`;
  popupCard.style.top = `${top}px`;

  popupWord.textContent = cleanWord;
  popupPos.textContent = "...";
  popupBody.innerHTML = `<p style="color:#71717a;margin:4px 0;">Searching offline dictionary...</p>`;
  popupCard.classList.remove("hidden");

  try {
    const def = await invoke<Definition>("lookup_word", { word: cleanWord });
    popupWord.textContent = def.word;
    popupPos.textContent = def.part_of_speech;
    popupBody.innerHTML = `
      <ol>
        ${def.meanings.map((m) => `<li>${escapeHtml(m)}</li>`).join("")}
      </ol>
    `;
  } catch (err) {
    popupPos.textContent = "";
    popupBody.innerHTML = `<p class="popup-error">${escapeHtml(String(err))}</p>`;
  }
}

document.addEventListener("pointerup", () => {
  setTimeout(handleSelection, 50);
});

function escapeHtml(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}

// Initial load
loadTypoSettings();
