# Konco Nembang

Overlay lirik transparan untuk Windows. Lirik ter-sinkron tampil di atas layar saat kerja atau main game (mode borderless), tembus klik, dan tidak mencuri fokus.

> Status: Tahap 1 (scaffold + overlay dengan lirik demo). Deteksi lagu dan pengambilan lirik menyusul di Tahap 2–3.

## Prasyarat
- Windows 10/11 (WebView2 sudah bawaan Windows 11)
- Node.js 24 LTS
- Rust stable (`rustup`) + Visual Studio 2022 Build Tools (workload "Desktop development with C++")

## Menjalankan
```bash
npm install
npm run tauri dev
```

Test:
```bash
npm test
```

## Shortcut sementara
| Shortcut | Fungsi |
|---|---|
| `Ctrl+Alt+L` | Kunci (click-through) / Edit (drag & resize) overlay |
| `Ctrl+Alt+H` | Tampilkan / sembunyikan overlay |
| `Ctrl+Alt+S` | Buka pengaturan |
| `Ctrl+Alt+Q` | Keluar |

## Struktur
```
overlay.html, settings.html   entry halaman untuk tiap window
src/lib/lrc.ts                parser LRC
src/lib/syncClock.ts          interpolasi posisi lagu + pencarian baris aktif
src/overlay/                  UI overlay (React)
src/settings/                 UI pengaturan (React)
src-tauri/src/overlay.rs      perilaku window overlay (click-through, no-activate, lock/edit)
spike/                        eksperimen Tahap 0 dan hasilnya (FINDINGS.md)
```

## Batasan yang diketahui
- Game dengan **exclusive fullscreen** tidak didukung. Gunakan mode borderless/windowed.
