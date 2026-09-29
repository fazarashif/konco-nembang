const { listen } = window.__TAURI__.event;

// Placeholder lines (not real lyrics) cycled on a timer to exercise the overlay.
const DEMO_LINES = [
  "Konco Nembang: tes overlay lirik",
  "Klik dan scroll harus tembus ke aplikasi di belakang",
  "Game tidak boleh kehilangan fokus",
  "Ctrl+Alt+L untuk mode edit (pindah dan resize)",
  "Ctrl+Alt+H sembunyikan, Ctrl+Alt+Q keluar",
  "Teks harus tetap terbaca di latar terang dan gelap",
];
const LINE_MS = 2500;

const prevEl = document.querySelector("#prev");
const currentEl = document.querySelector("#current");
const nextEl = document.querySelector("#next");

let index = 0;

function render() {
  const n = DEMO_LINES.length;
  prevEl.textContent = DEMO_LINES[(index - 1 + n) % n];
  nextEl.textContent = DEMO_LINES[(index + 1) % n];
  currentEl.textContent = DEMO_LINES[index];
  currentEl.classList.add("enter");
  requestAnimationFrame(() => requestAnimationFrame(() => currentEl.classList.remove("enter")));
}

render();
setInterval(() => {
  index = (index + 1) % DEMO_LINES.length;
  render();
}, LINE_MS);

listen("edit-mode", (event) => {
  document.body.classList.toggle("edit", event.payload === true);
});
