# Finder Auto-hide QA (F-8.4)

Status: **belum dijalankan** (butuh build Windows; logika inti sudah dites di `core/finder/appearance.rs`).

## Purpose
Bar Finder tersembunyi saat jendela menutupinya, muncul saat pointer menetap di tepi atas, dan tidak meninggalkan work area yang tersisa.

## Prerequisites
- Windows 11, build Nexora dengan Finder aktif, minimal dua monitor untuk langkah 6.
- Catat resolusi dan DPI tiap monitor.

## Test Steps
1. Nyalakan "Auto-hide" (FLB-02) dengan jendela biasa tidak menutupi bar. Bar tetap tampil.
2. Maksimalkan sebuah jendela. Bar harus tersembunyi.
3. Gerakkan pointer ke tepi atas dan tunggu. Bar harus muncul; jendela tidak boleh bergeser (work area tidak dipesan).
4. Buka menu bar lalu gerakkan pointer keluar. Bar tetap tampil selama menu terbuka, lalu tersembunyi lagi.
5. Matikan "Auto-hide" saat bar tersembunyi. Bar tampil dan work area dipesan lagi; jendela maksimal turun di bawah bar.
6. Ulangi 2 sampai 3 di monitor kedua; hanya bar di monitor itu yang berubah.
7. Nyalakan "Maximized windows affect appearance" (FLB-04): bar menjadi solid saat jendela maksimal ada di monitor yang sama, dan kembali ke kaca saat jendela dipulihkan.

## Expected Behavior
Sesuai langkah di atas; tidak ada kedipan dan tidak ada celah di atas jendela maksimal.

## Actual Behavior
_Belum diisi._

## Taskbar Recovery Check
- [ ] Taskbar is visible after exit
- [ ] Taskbar auto-hide setting is restored
- [ ] Work area is properly cleared

## Work Area Recovery Check
- [ ] Work area is restored to original state after switching auto-hide on and off
- [ ] No leftover reserved space at the top of any monitor
