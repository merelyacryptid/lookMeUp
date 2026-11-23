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
  chapters: EpubChapter[];
}

let currentBook: EpubBook | null = null;
let currentChapterIdx = 0;

const popupCard = document.querySelector<HTMLElement>("#popup-card")!;
const popupWord = document.querySelector<HTMLElement>("#popup-word")!;
const popupPos = document.querySelector<HTMLElement>("#popup-pos")!;
const popupBody = document.querySelector<HTMLElement>("#popup-body")!;
const popupClose = document.querySelector<HTMLElement>("#popup-close")!;
const openBtn = document.querySelector<HTMLElement>("#open-btn");
const bookTitleEl = document.querySelector<HTMLElement>("#book-title");
const readerContent = document.querySelector<HTMLElement>("#reader-content")!;
const prevBtn = document.querySelector<HTMLButtonElement>("#prev-btn");
const nextBtn = document.querySelector<HTMLButtonElement>("#next-btn");
const chapterLabel = document.querySelector<HTMLElement>("#chapter-label");

function renderChapter(idx: number) {
  if (!currentBook || !currentBook.chapters[idx]) return;
  currentChapterIdx = idx;

  const ch = currentBook.chapters[idx];
  readerContent.innerHTML = `<h1>${escapeHtml(ch.title)}</h1>${ch.content}`;

  if (chapterLabel) {
    chapterLabel.textContent = `${idx + 1} / ${currentBook.chapters.length}`;
  }
  if (prevBtn) prevBtn.disabled = idx <= 0;
  if (nextBtn) nextBtn.disabled = idx >= currentBook.chapters.length - 1;

  // Scroll reader to top on chapter change
  readerContent.parentElement?.scrollTo({ top: 0, behavior: "smooth" });
}

prevBtn?.addEventListener("click", () => {
  if (currentBook && currentChapterIdx > 0) {
    renderChapter(currentChapterIdx - 1);
  }
});

nextBtn?.addEventListener("click", () => {
  if (currentBook && currentChapterIdx < currentBook.chapters.length - 1) {
    renderChapter(currentChapterIdx + 1);
  }
});

// Close popup
function hidePopup() {
  popupCard.classList.add("hidden");
}

popupClose.addEventListener("click", hidePopup);

// Close on Escape
window.addEventListener("keydown", (e) => {
  if (e.key === "Escape") hidePopup();
});

// Close when clicking/tapping elsewhere in the document
document.addEventListener("pointerdown", (e) => {
  if (!popupCard.contains(e.target as Node)) {
    hidePopup();
  }
});

// Handle Touch and Mouse Word Selection
async function handleSelection() {
  const selection = window.getSelection();
  if (!selection || selection.isCollapsed) return;

  const rawText = selection.toString().trim();
  // Clean word: remove trailing punctuation
  const cleanWord = rawText.replace(/^[^\w]+|[^\w]+$/g, "");
  if (!cleanWord || cleanWord.length > 40) return;

  const range = selection.getRangeAt(0);
  const rect = range.getBoundingClientRect();

  // Position popup right above or below the tapped word
  const popupWidth = 320;
  const popupHeight = 180;
  
  let left = rect.left + rect.width / 2 - popupWidth / 2;
  // Keep inside screen bounds
  left = Math.max(12, Math.min(left, window.innerWidth - popupWidth - 12));

  let top = rect.top - popupHeight - 12;
  // If not enough room above, place below the word
  if (top < 60) {
    top = rect.bottom + 12;
  }

  popupCard.style.left = `${left}px`;
  popupCard.style.top = `${top}px`;

  // Render loading state
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

// Listen for selection release on both touch and mouse
document.addEventListener("pointerup", () => {
  // Slight delay allows browser to finalize word selection bounding rect
  setTimeout(handleSelection, 50);
});

// Native Open File Dialog for EPUBs
openBtn?.addEventListener("click", async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ name: "EPUB Documents", extensions: ["epub"] }],
    });

    if (selected) {
      const pathStr = Array.isArray(selected) ? selected[0] : selected;
      if (bookTitleEl) bookTitleEl.textContent = "Loading book...";

      const book = await invoke<EpubBook>("load_epub", { path: pathStr });
      currentBook = book;
      if (bookTitleEl) bookTitleEl.textContent = book.title;
      renderChapter(0);
    }
  } catch (err) {
    console.error("Open dialog error:", err);
    alert(`Could not open or render EPUB: ${err}`);
  }
});

function escapeHtml(str: string): string {
  const div = document.createElement("div");
  div.textContent = str;
  return div.innerHTML;
}
