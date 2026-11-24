# Finder Corner Close QA (F-10.2)

Status: **belum dijalankan** (butuh build Windows; hit-test dan daftar pengecualian sudah dites di `core/finder/actions.rs`).

## Purpose
Klik di pojok kanan atas menutup jendela foreground yang maksimal, dan tidak pernah menutup desktop, shell, atau Nexora.

## Prerequisites
- Windows 11, Finder aktif, "Top-right corner closes maximized window" (FLB-05) aktif.
- Siapkan Notepad dengan teks yang belum disimpan untuk langkah 2.

## Test Steps
1. Maksimalkan Notepad yang bersih lalu klik pojok kanan atas. Jendela menutup.
2. Maksimalkan Notepad dengan teks belum disimpan lalu klik pojok. Notepad harus menampilkan permintaan simpan seperti biasa (kita hanya mengirim `WM_CLOSE`), tidak menutup paksa.
3. Jendela yang tidak maksimal: klik pojok. Tidak terjadi apa-apa.
4. Fokus di desktop (klik wallpaper) lalu klik pojok. Tidak terjadi apa-apa.
5. Fokus di taskbar Windows lalu klik pojok. Tidak terjadi apa-apa.
6. Buka jendela Settings Nexora, maksimalkan, klik pojok. Tidak boleh menutup Nexora.
7. Di monitor kedua dengan posisi negatif (kiri atau atas dari monitor utama), klik pojok kanan atas monitor itu. Jendela maksimal di monitor itu yang menutup.
8. Matikan FLB-05 dan ulangi 1. Tidak terjadi apa-apa.

## Expected Behavior
Sesuai langkah di atas; pojok tidak mencuri klik dari tombol tutup jendela maksimal (tombol itu ada di area yang sama, jadi pastikan hanya satu aksi berjalan per klik).

## Actual Behavior
_Belum diisi._
