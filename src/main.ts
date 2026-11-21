import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface Definition {
  word: string;
  part_of_speech: string;
  meanings: string[];
}

type LookupPayload = { Ok: Definition } | { Err: string };

const wordTitleEl = document.querySelector<HTMLElement>("#word-title");
const wordPosEl = document.querySelector<HTMLElement>("#word-pos");
const cardBodyEl = document.querySelector<HTMLElement>("#card-body");
const closeBtn = document.querySelector<HTMLElement>("#close-btn");

async function hidePopup() {
  try {
    await invoke("hide_window");
  } catch (e) {
    console.error("Failed to hide window:", e);
  }
}

// Close on button click
closeBtn?.addEventListener("click", hidePopup);

// Close on Escape key
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") {
    hidePopup();
  }
});

// Auto-close on blur (when user clicks anywhere outside the popup)
window.addEventListener("blur", () => {
  hidePopup();
});

// Listen for definition payload from Rust
listen<LookupPayload>("show-definition", (event) => {
  if (!wordTitleEl || !wordPosEl || !cardBodyEl) return;

  const payload = event.payload;
  if ("Ok" in payload) {
    renderDefinition(payload.Ok);
  } else if ("Err" in payload) {
    renderError(payload.Err);
  } else {
    renderError("No definition returned.");
  }
});

function renderDefinition(def: Definition) {
  if (!wordTitleEl || !wordPosEl || !cardBodyEl) return;
  wordTitleEl.textContent = def.word;
  wordPosEl.textContent = def.part_of_speech;
  cardBodyEl.innerHTML = `
    <ol class="meanings-list">
      ${def.meanings.map((m) => `<li>${escapeHtml(m)}</li>`).join("")}
    </ol>
  `;
}

function renderError(err: string) {
  if (!wordTitleEl || !wordPosEl || !cardBodyEl) return;
  wordTitleEl.textContent = "Lookup";
  wordPosEl.textContent = "";
  cardBodyEl.innerHTML = `<p class="error-msg">${escapeHtml(err)}</p>`;
}

function escapeHtml(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}
