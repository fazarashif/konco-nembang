# Tahap 0: Hasil Spike

## Eksperimen 1: Deteksi lagu via GSMTC (`gsmtc-probe.ps1`)
Dites 2026-09-29 dengan YouTube Music (PWA Chrome) + YouTube di Opera GX.

- YT Music terbaca sebagai `Chrome._crx_cinhimbnkkghhklpknlkffjgod`, metadata bersih (title, artist, album, durasi).
- YouTube biasa (Opera GX) terbaca, tapi `Artist` = nama channel, jadi normalisasi judul memang dibutuhkan.
- Bisa ada banyak session sekaligus; `GetCurrentSession()` tidak selalu yang sedang diputar, jadi pilih yang berstatus `Playing` dulu.
- Timeline hanya di-update **~1x per 20 detik**. Interpolasi `Position + (now - LastUpdatedTime)` akurat: koreksi saat update ~100 ms, cukup untuk sync per baris.
- LRCLIB `/api/get` dengan title+artist+album+durasi langsung mengembalikan `syncedLyrics` (65 baris). `/api/search` mengembalikan banyak versi (lyric video, live), jadi matching harus pakai durasi.

## Eksperimen 2: Overlay Tauri (`overlay-spike/`)
- Tauri 2 + window `transparent`, `decorations: false`, `alwaysOnTop`, `skipTaskbar`, `shadow: false` → latar benar-benar transparan di Windows 11.
- `set_ignore_cursor_events(true)` memberi `WS_EX_TRANSPARENT | WS_EX_LAYERED` (click-through).
- `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` ditambahkan manual via `windows-sys` dan harus di-apply ulang setiap kali toggle click-through.
- Diverifikasi dengan `GetWindowLongPtr`: TOPMOST, TRANSPARENT, LAYERED, TOOLWINDOW, NOACTIVATE semuanya aktif; overlay tidak menjadi foreground window.
- Text-shadow outline terbaca di atas latar gelap.
- Performa (debug build, animasi ganti baris tiap 2,5 detik): CPU **0,13%** (16 core), RAM private **~171 MB** (1 proses app + 6 proses WebView2).
  Target awal <80 MB tidak realistis dengan WebView2.
  Release build v0.1.0 (overlay saja, Settings/Search dibuat saat dibuka): ~231 MB private total; app 10 MB, GPU process ~84 MB, browser process ~48 MB, renderer ~35–40 MB per halaman. CPU 0,19%.

## Tes manual (lolos semua, dikonfirmasi user 2026-09-29)
- [x] Klik & scroll tembus ke aplikasi di belakang overlay
- [x] Di atas game borderless: game tetap fokus, overlay tetap terlihat
- [x] Ctrl+Alt+L mode edit: drag pindah, resize dari tepi
- [x] Keterbacaan di atas latar terang
