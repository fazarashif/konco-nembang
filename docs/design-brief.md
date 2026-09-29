# Brief Desain: Konco Nembang — Overlay Lirik Transparan untuk Windows

> Brief desain. Hasilnya (tokens, prototype, aset) disimpan di `docs/design/`.

## Tentang produk
Konco Nembang (bahasa Jawa: "teman bernyanyi") adalah aplikasi desktop Windows 10/11 yang otomatis mendeteksi lagu yang sedang diputar (Spotify, YouTube Music, YouTube di browser), mengambil lirik yang tersinkron, lalu menampilkannya sebagai **overlay transparan yang selalu di atas layar dan tembus klik**. Pengguna tetap bisa bekerja atau main game (mode borderless) sambil membaca lirik tanpa terganggu.

Prinsip utama: **terbaca dalam sekilas, tapi tidak mengganggu.** Overlay adalah tamu di layar pengguna. Tidak boleh mencuri perhatian, tidak boleh ramai, dan tidak boleh menutupi UI game lebih dari yang perlu.

Target pengguna: orang Indonesia yang suka musik (termasuk lagu Jawa/dangdut/pop Indonesia), bekerja di depan komputer, atau gamer PC. Bahasa UI utama Bahasa Indonesia, dengan dukungan Bahasa Inggris.

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
- **Status tanpa lirik**: judul + artis + posisi (▶ 1:23 / 4:57) ditambah catatan: "Mencari lirik…", "Lirik tidak ditemukan", "♪ Instrumental", "Gagal memuat lirik".
- **Lagu di-pause**: apakah overlay meredup, menampilkan ikon pause, atau disembunyikan?
- **Baris sangat panjang**: wrap ke 2 baris atau diperkecil? Tentukan aturannya.
- **Mode Edit** (pengguna menekan Ctrl+Alt+L untuk memindah/resize): border, handle resize, dan badge info kecil berisi instruksi serta sumber lirik ("LRCLIB: Judul — Artis").
- **Tidak ada lagu**: overlay tidak terlihat sama sekali, kecuali di Mode Edit.
- Alignment kiri / tengah / kanan.

**Preset gaya** (3–5 buah) yang bisa dipilih pengguna, misalnya: *Minimal* (teks saja + outline), *Pill* (latar kapsul semi-transparan), *Karaoke* (baris aktif diberi warna aksen), dan *Subtle* (kecil, opacity rendah untuk kerja). Setiap preset harus terdefinisi lewat token yang sama supaya bisa diubah dari Pengaturan.

### 2. Window Pengaturan (520×640 px, bisa di-resize)
Mode terang dan gelap. Berisi **live preview overlay** kecil di bagian atas yang langsung berubah saat pengaturan diubah. Bagian-bagiannya:
- **Tampilan**: preset gaya, font, ukuran teks, warna baris aktif & tidak aktif, opacity teks, opacity latar, outline/shadow, jumlah baris (1/2/3), alignment, animasi on/off.
- **Perilaku**: sembunyikan otomatis saat pause / saat tidak ada lirik, sumber media (Otomatis / Spotify saja / Browser saja).
- **Sinkronisasi**: offset global (slider ± detik), info offset lagu saat ini + tombol reset.
- **Shortcut**: daftar hotkey yang bisa diubah (klik lalu tekan kombinasi baru): Kunci/Edit overlay, Tampilkan/Sembunyikan, Offset +/-, Buka Pengaturan.
- **Umum**: jalankan saat Windows start, bahasa (Indonesia/English), cek pembaruan.
- **Tentang**: versi, atribusi LRCLIB, catatan privasi singkat ("hanya judul & artis yang dikirim ke LRCLIB").

### 3. Dialog Cari Lirik Manual
Dipakai saat lirik salah atau tidak ditemukan. Isinya: kolom pencarian (terisi otomatis dengan judul & artis yang terdeteksi), daftar hasil (judul, artis, album, durasi + selisih durasi dengan lagu yang diputar, badge "Tersinkron"/"Teks saja"), panel preview lirik, tombol "Pakai lirik ini", dan area **drag & drop file .lrc** untuk import lirik sendiri. Sertakan state kosong, loading, dan error.

### 4. Toast / OSD hotkey
Notifikasi kecil yang muncul sekitar 1,2 detik di dekat overlay saat hotkey ditekan, misalnya "Overlay dikunci", "Mode edit", "Offset +0,25 dtk (lagu ini)". Harus konsisten dengan gaya overlay dan tidak mengganggu.

### 5. Onboarding (pertama kali dibuka)
1–3 layar singkat: cara kerja (putar lagu → lirik muncul), daftar hotkey penting, dan tips "gunakan mode borderless/windowed di game, karena exclusive fullscreen tidak didukung".

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
