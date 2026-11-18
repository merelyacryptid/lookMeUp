import { invoke } from "@tauri-apps/api/core";

interface Definition {
  word: string;
  part_of_speech: string;
  meanings: string[];
}



let greetInputEl: HTMLInputElement | null;
let greetMsgEl: HTMLElement | null;

async function lookup() {
  if (!greetInputEl || !greetMsgEl) return;
  const word = greetInputEl.value.trim();

  try {
    const def = await invoke<Definition>("lookup_word", { word });
    greetMsgEl.innerHTML = `
      <strong>${def.word}</strong> <em>(${def.part_of_speech})</em>
      <ul>${def.meanings.map(m => `<li>${m}</li>`).join("")}</ul>
    `;
  } catch (err) {
    greetMsgEl.textContent = `Error: ${err}`;
  }
}

window.addEventListener("DOMContentLoaded", () => {
  greetInputEl = document.querySelector("#greet-input");
  greetMsgEl = document.querySelector("#greet-msg");
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    lookup();
  });
});
