# Konco Nembang

Overlay lirik transparan untuk Windows. Aplikasi mendeteksi lagu yang sedang diputar (Spotify, YouTube Music, YouTube di browser), mengambil lirik ter-sinkron dari [LRCLIB](https://lrclib.net), lalu menampilkannya di atas layar: tembus klik dan tidak mencuri fokus, jadi aman dipakai saat kerja atau main game (mode borderless).

> Status: Tahap 6 (iklan, offline, cari lirik manual). Tampilan final menunggu hasil desain (`docs/design-brief.md`).

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

```bash
cd src-tauri && cargo test
```

## Cara pakai
App berjalan di **system tray**. Klik kiri ikon tray untuk membuka Settings; klik kanan untuk menu (show/hide overlay, move/resize, search lyrics, settings, quit).

Lirik salah atau tidak ketemu? Buka **Search lyrics…** dari tray (atau Settings → Sync): cari di LRCLIB, pilih hasil, atau drag & drop file `.lrc`/`.txt`. Pilihan manual disimpan untuk lagu itu; **Reset to automatic** mengembalikannya ke pencarian otomatis.

Iklan (Spotify Free, pre-roll YouTube) dan klip di bawah 40 detik diabaikan. Saat offline, lirik yang sudah pernah dimuat tetap tampil dari cache, dan pencarian baru dicoba ulang otomatis.

Shortcut default (bisa diubah di Settings → Shortcuts):

| Shortcut | Fungsi |
|---|---|
| `Ctrl+Alt+L` | Mode edit: pindah & resize overlay (tekan lagi untuk mengunci) |
| `Ctrl+Alt+H` | Tampilkan / sembunyikan overlay |
| `Ctrl+Alt+S` | Buka Settings |
| `Ctrl+Alt+=` | Lirik lebih awal (untuk lagu ini) |
| `Ctrl+Alt+-` | Lirik lebih lambat (untuk lagu ini) |
| `Ctrl+Alt+0` | Reset offset lagu ini |

Offset per lagu disimpan berdasarkan judul + artis + durasi, jadi video klip dan versi audio lagu yang sama punya offset masing-masing.

## Data yang disimpan
Semua di `%APPDATA%\com.konconembang.desktop\`:
- `settings.json` pengaturan
- `overlay.json` posisi & ukuran overlay
- `offsets.json` offset per lagu
- `lyrics\` cache lirik

## Struktur
```
overlay.html, settings.html,
search.html                   entry halaman untuk tiap window
src/lib/                      logika bersama: parser LRC, sync clock, IPC, auto-hide, shortcut
src/overlay/                  UI overlay (React)
src/settings/                 UI Settings (React)
src/search/                   UI Search Lyrics (React)
src-tauri/src/overlay.rs      window overlay (click-through, no-activate, lock/edit, posisi)
src-tauri/src/media/          deteksi lagu via Windows GSMTC
src-tauri/src/lyrics/         LRCLIB, normalisasi judul, cache, offset per lagu
src-tauri/src/settings.rs     pengaturan + penyimpanan
src-tauri/src/hotkeys.rs      hotkey global yang bisa diubah
src-tauri/src/tray.rs         ikon & menu tray
spike/                        eksperimen Tahap 0 dan hasilnya (FINDINGS.md)
docs/design-brief.md          brief desain
```

## Batasan yang diketahui
- Game dengan **exclusive fullscreen** tidak didukung. Gunakan mode borderless/windowed.
- Lirik video klip (dengan intro/adegan tambahan) bisa meleset dari versi audio. Atur dengan offset per lagu.
