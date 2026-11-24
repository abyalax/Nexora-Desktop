# Finder Shut-down Confirmation QA (F-9.2)

Status: **belum dijalankan** (butuh build Windows; keputusan "hanya Restart dan Shut Down" sudah dites di `core/finder/actions.rs`).

## Purpose
Dialog konfirmasi bertema muncul sebelum Restart atau Shut Down dari menu logo, dan bisa dijawab dengan keyboard.

## Prerequisites
- Windows 11, Finder aktif, "Shut-down confirmation" (FLB-03) aktif.
- Simpan semua pekerjaan; langkah 3 benar-benar mematikan atau me-restart mesin, jadi uji di VM bila memungkinkan.

## Test Steps
1. Pilih Sleep dan Lock dari menu logo. Keduanya langsung berjalan tanpa dialog.
2. Pilih Restart. Dialog muncul dengan tema yang sama dengan bar (terang, gelap, kaca).
3. Tekan Esc. Dialog tertutup dan tidak terjadi apa-apa. Ulangi dengan klik tombol Batal.
4. Pilih Shut Down, tekan Enter. Mesin harus mati. (Gunakan VM.)
5. Matikan FLB-03 lalu pilih Restart. Tidak ada dialog.
6. Tekan tombol lain (misalnya spasi atau huruf) saat dialog terbuka. Tidak ada efek.

## Expected Behavior
Sesuai langkah di atas. Dialog tidak muncul di balik jendela lain dan tidak membuat bar kehilangan fokus secara aneh.

## Actual Behavior
_Belum diisi._
