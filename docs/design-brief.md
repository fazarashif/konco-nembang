# Brief Desain: Konco Nembang — Overlay Lirik Transparan untuk Windows

> Brief desain. Hasilnya (tokens, prototype, aset) disimpan di `docs/design/`.

## Tentang produk
Konco Nembang (bahasa Jawa: "teman bernyanyi") adalah aplikasi desktop Windows 10/11 yang otomatis mendeteksi lagu yang sedang diputar (Spotify, YouTube Music, YouTube di browser), mengambil lirik yang tersinkron, lalu menampilkannya sebagai **overlay transparan yang selalu di atas layar dan tembus klik**. Pengguna tetap bisa bekerja atau main game (mode borderless) sambil membaca lirik tanpa terganggu.

Prinsip utama: **terbaca dalam sekilas, tapi tidak mengganggu.** Overlay adalah tamu di layar pengguna. Tidak boleh mencuri perhatian, tidak boleh ramai, dan tidak boleh menutupi UI game lebih dari yang perlu.

Target pengguna: orang Indonesia yang suka musik (termasuk lagu Jawa/dangdut/pop Indonesia), bekerja di depan komputer, atau gamer PC. **Semua teks UI dalam bahasa Inggris** (label, status, menu, toast, pengaturan); hanya lirik yang tampil apa adanya.

## Batasan teknis (wajib diikuti)
- Stack: Tauri 2 + React + TypeScript, styling dengan **CSS biasa** (tanpa Tailwind/UI library berat). Render di WebView2 (Chromium).
- Overlay adalah **window transparan**. `backdrop-filter`/blur **tidak bisa mem-blur aplikasi atau game di belakangnya**, jadi jangan mengandalkan efek frosted glass. Keterbacaan harus dicapai dengan outline/shadow teks, atau "pill" latar semi-transparan berwarna solid.
- Animasi hanya `opacity` dan `transform`, durasi ≤ 250 ms, easing lembut. Harus ada varian tanpa animasi (reduced motion). Overlay berjalan saat main game, jadi hindari efek yang boros GPU (glow besar, blur, partikel).
- Font harus **di-bundle lokal** (app bisa offline) dengan lisensi terbuka (OFL/Apache). Harus mendukung huruf Latin lengkap termasuk diakritik, dengan fallback untuk lirik Jepang/Korea/Mandarin.
- Window Pengaturan mengikuti tema Windows (terang/gelap), bernuansa Windows 11 (Fluent-ish: rounded 8px, spacing lega), tapi tetap punya identitas sendiri.
- Gunakan **lirik placeholder buatan sendiri** di semua mockup, bukan lirik lagu asli (hak cipta).

## Kondisi overlay saat ini (untuk dirapikan)
- Ukuran default 900×170 px, bisa di-resize, posisi default di tengah bawah layar.
- 3 baris: baris sebelumnya (18px, putih, opacity 55%), **baris aktif** (34px, bold, putih), baris berikutnya (18px, opacity 55%).
- Keterbacaan lewat text-shadow hitam 4 lapis. Font Segoe UI.
- Baris aktif masuk dengan fade + geser 8px ke atas, 250 ms.

## Yang saya butuhkan

### 1. Overlay lirik (prioritas tertinggi)
Desain semua state berikut, masing-masing ditampilkan di atas **4 jenis latar**: desktop gelap, halaman web putih terang, screenshot game yang ramai/berwarna, dan video/latar dengan kontras campuran.
- **Lirik tersinkron**: baris sebelumnya / aktif / berikutnya, termasuk varian 1 baris, 2 baris (aktif + berikutnya), dan 3 baris.
- **Intro** (sebelum lirik pertama mulai): menampilkan judul lagu.
- **Jeda instrumental** di tengah lagu (sekarang ditandai "♪").
- **Lirik tidak tersinkron (perkiraan)**: perlu penanda halus bahwa waktunya hanya perkiraan.
- **Status tanpa lirik**: judul + artis + posisi (▶ 1:23 / 4:57) ditambah catatan: "Searching for lyrics…", "No lyrics found", "♪ Instrumental", "Couldn't load lyrics".
- **Lagu di-pause**: saat ini overlay memudar setelah 3 detik (bisa diatur). Usulkan transisinya, dan apakah perlu indikator pause sebelum memudar.
- **Baris sangat panjang**: saat ini baris aktif dibungkus maksimal 2 baris, baris konteks dipotong "…". Rapikan aturannya.
- **Mode Edit** (pengguna menekan Ctrl+Alt+L untuk memindah/resize): border, handle resize, dan badge info kecil berisi instruksi serta sumber lirik ("LRCLIB: Title — Artist").
- **Tidak ada lagu**: overlay tidak terlihat sama sekali, kecuali di Mode Edit ("Nothing is playing").
- Alignment kiri / tengah / kanan, dan opsi latar gelap semi-transparan di belakang lirik (opacity 0–90%).
- Ukuran teks mengikuti ukuran window overlay (diskalakan terhadap tinggi 170px), dikali pengaturan "Text size" 50–250%.

**Preset gaya** (3–5 buah) yang bisa dipilih pengguna, misalnya: *Minimal* (teks saja + outline), *Pill* (latar kapsul semi-transparan), *Karaoke* (baris aktif diberi warna aksen), dan *Subtle* (kecil, opacity rendah untuk kerja). Setiap preset harus terdefinisi lewat token yang sama supaya bisa diubah dari Pengaturan.

### 2. Window Settings (520×640 px, bisa di-resize)
Mode terang dan gelap. Versi fungsionalnya **sudah jadi** (tampilan sederhana), jadi desain ulang struktur yang ada ini dan tambahkan **live preview overlay** kecil di bagian atas. Semua label dalam bahasa Inggris:
- **Appearance**: Lines shown (1/2/3), Text size (50–250%), Alignment (Left/Center/Right), Text opacity, Background (Off–90%), Active line color, Animations on/off. Tambahkan pemilih **preset gaya** dan font.
- **Behavior**: Hide when paused (off / 1–30 s), Hide when no lyrics (off / 1–30 s), Follow music from (Any app / Spotify / Browsers).
- **Sync**: Global offset (slider ±5 s), Hotkey step (0.1 / 0.25 / 0.5 s), This track (judul lagu + tombol Later / Earlier / Reset + nilai offset).
- **Shortcuts**: 6 hotkey yang bisa direkam ulang (klik lalu tekan kombinasi; Esc batal, Backspace hapus): Move / resize overlay, Show / hide overlay, Open settings, Show lyrics earlier, Show lyrics later, Reset track offset. Sertakan state "recording" dan pesan error untuk shortcut bentrok.
- **General**: Start with Windows, Overlay position (Reset).
- **About**: versi, atribusi LRCLIB, catatan privasi ("Only the title, artist, album and duration of the playing track are sent to LRCLIB.").
- Banner peringatan/error di atas (mis. shortcut sudah dipakai aplikasi lain).

### 3. Dialog Cari Lirik Manual
Dipakai saat lirik salah atau tidak ditemukan. Isinya: kolom pencarian (terisi otomatis dengan judul & artis yang terdeteksi), daftar hasil (judul, artis, album, durasi + selisih durasi dengan lagu yang diputar, badge "Synced"/"Text only"), panel preview lirik, tombol "Use these lyrics", dan area **drag & drop file .lrc** untuk import lirik sendiri. Sertakan state kosong, loading, dan error.

### 4. Toast / OSD hotkey
Sudah ada versi sederhana: kapsul gelap di tengah atas overlay selama 1,6 detik, misalnya "Lyrics 0.25 s earlier for this track", "Lyrics offset reset for this track", "No track playing". Rapikan agar konsisten dengan gaya overlay dan tidak mengganggu.

### 4b. Menu tray
Menu native Windows (hanya teks): Show / hide overlay, Move / resize overlay, Settings…, Quit Konco Nembang. Klik kiri ikon tray membuka Settings. Tooltip menampilkan lagu yang sedang diputar.

### 5. Onboarding (pertama kali dibuka)
1–3 layar singkat (bahasa Inggris): cara kerja (putar lagu → lirik muncul), daftar hotkey penting, dan tips bahwa game harus memakai mode borderless/windowed karena exclusive fullscreen tidak didukung.

### 6. Branding & ikon
- Logo/ikon aplikasi yang mencerminkan "teman bernyanyi" (boleh ada sentuhan lokal Jawa/Indonesia, tapi tetap modern dan sederhana).
- Ikon app dalam ukuran 16, 24, 32, 48, 256 px (PNG) plus SVG master.
- **Ikon tray** monokrom yang tetap terbaca di taskbar terang dan gelap, dengan varian status: normal, overlay tersembunyi, mode edit.
- Gambar installer NSIS: header 150×57 px dan sidebar 164×314 px.
- Palet warna brand + satu warna aksen (dipakai untuk preset Karaoke dan elemen aktif di Pengaturan).

## Format hasil yang saya harapkan
1. **Design tokens** sebagai CSS custom properties (`--kn-*`): warna, tipografi (font, ukuran, weight, line-height), spacing, radius, shadow/outline teks, durasi & easing animasi. Sertakan nilai untuk tema terang/gelap dan untuk tiap preset overlay.
2. **Prototype HTML/CSS** untuk overlay (semua state di atas) dan window Pengaturan. Struktur yang mudah dipindahkan ke komponen React (satu komponen per bagian, class name yang jelas, tanpa framework CSS).
3. **Spesifikasi komponen**: ukuran dalam px, padding, perilaku hover/focus/disabled, dan aturan teks panjang.
4. **Aset**: SVG logo, ikon app & tray, gambar installer.
5. Catatan singkat alasan desain untuk keputusan penting (keterbacaan, preset default, perilaku saat pause).
