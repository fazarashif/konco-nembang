# Konco Nembang

Overlay lirik transparan untuk Windows. Aplikasi mendeteksi lagu yang sedang diputar (Spotify, YouTube Music, YouTube di browser), mengambil lirik ter-sinkron dari [LRCLIB](https://lrclib.net), lalu menampilkannya di atas layar: tembus klik dan tidak mencuri fokus, jadi aman dipakai saat kerja atau main game (mode borderless).

**Install:** unduh installer dari [Releases](https://github.com/fazarashif/konco-nembang/releases/latest) dan ikuti [panduan instalasi](docs/INSTALL.md) (termasuk cara melewati peringatan SmartScreen).

## Cara pakai
App berjalan di **system tray**. Klik kiri ikon tray untuk membuka Settings; klik kanan untuk menu (show/hide overlay, move/resize, search lyrics, settings, quit).

Shortcut default (bisa diubah di Settings → Shortcuts):

| Shortcut | Fungsi |
|---|---|
| `Ctrl+Alt+L` | Mode edit: pindah & resize overlay (tekan lagi untuk mengunci) |
| `Ctrl+Alt+H` | Tampilkan / sembunyikan overlay |
| `Ctrl+Alt+S` | Buka Settings |
| `Ctrl+Alt+=` | Lirik lebih awal (untuk lagu ini) |
| `Ctrl+Alt+-` | Lirik lebih lambat (untuk lagu ini) |
| `Ctrl+Alt+0` | Reset offset lagu ini |

- **Gaya overlay:** Minimal, Pill, Karaoke, Subtle (Settings → Appearance → Style).
- **Offset per lagu** disimpan berdasarkan judul + artis + durasi, jadi video klip dan versi audio lagu yang sama punya offset masing-masing.
- **Lirik salah atau tidak ketemu?** Buka **Search lyrics…** dari tray: cari di LRCLIB, pilih hasil, atau drag & drop file `.lrc`/`.txt`. **Reset to automatic** mengembalikan ke pencarian otomatis.
- **Iklan** (Spotify Free, pre-roll YouTube) dan klip di bawah 40 detik diabaikan. Saat offline, lirik dari cache tetap tampil dan pencarian baru dicoba ulang otomatis.
- **Update otomatis** dari GitHub Releases (ditandatangani); cek manual di Settings → About.

## Data yang disimpan
Semua di `%APPDATA%\com.konconembang.desktop\`: `settings.json`, `overlay.json` (posisi overlay), `offsets.json` (offset per lagu), dan `lyrics\` (cache lirik).

## Development

Prasyarat: Windows 10/11, Node.js 24 LTS, Rust stable, Visual Studio 2022 Build Tools (workload "Desktop development with C++").

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

Build installer lokal (hasil di `src-tauri/target/release/bundle/nsis/`):
```bash
npm run tauri build
```
Untuk artefak update yang ditandatangani, set `TAURI_SIGNING_PRIVATE_KEY` (path atau isi kunci privat) dan `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` sebelum build.

## Rilis

1. Naikkan versi di `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, dan `package.json`.
2. Commit, lalu buat dan push tag yang sama, mis. `v0.2.0`:
   ```bash
   git tag v0.2.0
   ```
   ```bash
   git push origin v0.2.0
   ```
3. Workflow **Release** membangun installer, menandatangani update, dan membuat **draft release** berisi `latest.json`.
4. Periksa draft di GitHub, lalu **Publish**. Setelah dipublikasikan, app yang sudah terpasang akan menemukan update-nya.

Workflow butuh dua repository secret (Settings → Secrets and variables → Actions):

| Secret | Isi |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | isi file kunci privat updater |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | password kunci tersebut |

Kunci publiknya ada di `plugins.updater.pubkey` di `tauri.conf.json`. Kalau kunci privat hilang, app yang sudah terpasang tidak bisa menerima update lagi, jadi simpan cadangannya di tempat aman.

## Struktur
```
overlay.html, settings.html,
search.html                   entry halaman untuk tiap window
src/styles/kn-tokens.css      design tokens (warna, tipografi, preset overlay)
src/lib/                      logika bersama: parser LRC, sync clock, IPC, auto-hide, shortcut
src/overlay/                  UI overlay (React)
src/settings/                 UI Settings (React)
src/search/                   UI Search Lyrics (React)
src-tauri/src/overlay.rs      window overlay (click-through, no-activate, lock/edit, posisi)
src-tauri/src/media/          deteksi lagu via Windows GSMTC
src-tauri/src/lyrics/         LRCLIB, normalisasi judul, cache, offset per lagu
src-tauri/src/settings.rs     pengaturan + penyimpanan + migrasi
src-tauri/src/hotkeys.rs      hotkey global yang bisa diubah
src-tauri/src/tray.rs         ikon & menu tray (ikut tema taskbar)
src-tauri/src/updates.rs      update otomatis
.github/workflows/            CI dan rilis
docs/INSTALL.md               panduan instalasi (English)
docs/design/                  sumber desain (tokens, aset, catatan)
spike/                        eksperimen awal dan hasilnya (FINDINGS.md)
```

## Batasan yang diketahui
- Game dengan **exclusive fullscreen** tidak didukung. Gunakan mode borderless/windowed.
- Lirik video klip (dengan intro/adegan tambahan) bisa meleset dari versi audio. Atur dengan offset per lagu.
- Pemakaian RAM sekitar 230 MB saat hanya overlay yang aktif; hampir semuanya milik WebView2 (proses GPU untuk window transparan sekitar 85 MB). Proses app sendiri sekitar 10 MB. Window Settings/Search dibuat saat dibuka dan dilepas saat ditutup.
- Installer belum ditandatangani (code signing), jadi Windows SmartScreen menampilkan peringatan saat pertama kali dijalankan.
