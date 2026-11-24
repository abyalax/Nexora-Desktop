This is TODO based on ../TODO.md, list ini khusus untuk task yang windows platform depend

## 1. Baseline, CI, dan alat uji (M1)
- [x] **BASE-01**: jalankan empat perintah DoD di Windows 11 pada HEAD, catat yang gagal, dan perbaiki hanya yang memblokir build.
- [x] **BASE-02**: sambungkan atau hapus enam file orphan `app/dock/{popups,magnify,surfaces,items,geometry,prefs_sync}.rs`, lalu putuskan nasib `DockConfig` (`behavior.rs`).
- [x] **BASE-04**: job Windows di CI (fmt, clippy, test, check untuk workspace).
- [x] **BASE-08**: `cargo test -p nexora-platform -- --ignored` di runner Windows.
- [x] **BASE-09**: flag `--settings-page <id>` dan smoke run per halaman.
- [x] **BASE-10**: `docs/qa/TEMPLATE.md`, dengan cek pemulihan taskbar dan work area.
- [x] **BASE-11**: `tools/bench/m1-verify.ps1` untuk baseline memori idle, hasilnya dicatat di `docs/bench.md`; diulang tiap akhir milestone (QA-03).
- [x] **BASE-12**: audit semua baris ✔/~/◐ untuk Co/UI/Fx.

## 2. Jendela Settings (app/ui)
- [x] **WIN-01…06**: slider lebar, select 6+ opsi, row warna, row list, sidebar bergrup dengan enum `Page`, dan state row nonaktif.
- [x] **UI + Fx per baris**: sekitar 150 baris M2 (DLB, DLP, DAD, DMN, DPV, STG, FLB, FNO, FTD, FAU, FDI, FHW, FBA, IAC, APP, TRY, SCR, GEN, WAN) ditambahkan ke settings_win.rs; entri `docs/qa/` untuk perilaku jendela/OS masih perlu dibuat.
- [x] **Wiring `ProgramLists`**: baca/tulis `lists.cfg` secara atomik, snapshot `Arc` di `prefs::lists()`, dan row Settings untuk WAN-04/05, DLP-05, DMN-09.
- [x] **DAD-11**: `run()` di `hotcorners.rs` dan `hot_label` di `settings_win.rs` ikut berubah kalau varian `HotAction` ditambah (D-16.2).

## 3. Dynamic Island (M3)
- [x] **I-14.3**: tab/zona tersembunyi di app, plus QA `docs/qa/island-activities.md` (kodenya sudah ditulis).
- [x] **I-14.4**: row UI untuk 7 setting Activities (kodenya sudah ditulis).
- [ ] **I-12.3**: ukur bahwa tidak ada wake-up selain tick menit saat idle (butuh BASE-11).
- [x] **I-15.3**: gambar mata dan pelacakan pointer - OUT per ADR-0007, revisit setelah M3.
- [ ] **I-13**: notifikasi di Island, setelah spike F-12.0.

## 4. Dock (M4)
- [x] **D-1.4/1.5**: label selalu tampil, dan bounce saat launch.
- [ ] **D-3.2**: drag diabaikan saat terkunci (QA).
- [ ] **D-6.2/6.4**: popup preview membaca ukuran dan delay secara live; putuskan apakah "native preview" itu DWM thumbnail.
- [ ] **D-7.5/7.7/7.8**: minimize biasa saat efek mati (QA), konten jendela live (`platform/wins/thumbnail.rs`), dan semantik "minimize into icon" (QA).
- [ ] **D-8.2/8.5/8.6**: enumerasi shortcut Start Menu dan Desktop, Launchpad per monitor, serta di atas jendela vs sembunyikan jendela dan blur live.
- [ ] **D-10.1/10.5/10.6/10.7/10.8**: pindahkan aturan visibilitas dari `app/dock/visibility.rs` ke core (butuh build untuk memastikan perilaku tidak berubah), hitung overlap (`wins/overlap.rs`), efek hide Move/Fade, reserve work area (QA: tidak ada sisa setelah ganti mode), dan ganti mode saat runtime.
- [ ] **D-11.5…11.8**: penempatan per sisi layar, render vertikal, anchor tooltip/preview/popup yang berbalik per sisi, dan target minimize dari posisi baru.
- [ ] **D-12.2/12.3/12.5**: daftar monitor dengan id stabil, rebuild saat hotplug (QA), dan select dari display yang terdeteksi.
- [ ] **D-13.4**: jendela hit-test tak terlihat (layered, no-activate, topmost).
- [ ] **D-14.3/14.4**: plate dan efek cahaya ikon.
- [ ] **D-15.3**: renderer refleksi, dengan cek footprint.
- [ ] **D-16.2**: aksi platform untuk display sleep, Start Menu, dan Windows widget.
- [ ] **D-17.2/17.3**: daftar isi folder, popup grid, double-click ke Explorer.

## 5. Finder / Menu Bar (M5)
- [ ] **F-8.3/8.4**: deteksi jendela maksimal per monitor, reveal di tepi atas, dan pelepasan work area saat tersembunyi (QA).
- [ ] **F-9**: dialog konfirmasi shutdown bertema (Enter/Esc).
- [ ] **F-10**: kirim `WM_CLOSE` ke jendela foreground yang maksimal (QA).
- [ ] **F-11.0** (spike): cara enumerasi dan hosting ikon tray; ini memblokir F-11.5…7.
- [ ] **F-11.5…7**: enumerasi dan event perubahan tray, render, forward klik, drag reorder, plus row TRY-01…04, FLB-06, FLB-08.
- [ ] **F-12.0** (spike): API Windows untuk membaca notifikasi di app tanpa package identity; ini memblokir F-12.2…4 dan I-13.
- [ ] **F-12.2/12.3**: listener notifikasi di platform dan menu Finder.
- [ ] **F-13…F-14**: menu audio (`platform/audio.rs`) dan brightness (`platform/brightness.rs`).
- [ ] **F-16.4**: ganti power mode/plan dan event AC/baterai (QA).
- [ ] **F-17/F-18**: media di bar, daftar layout keyboard, nama dan avatar akun.

## 6. Halaman bersama (M6)
- [ ] **Start on boot**: Registry (sudah ada di `platform/startup.rs`) dan Scheduled Task; tes buat/hapus entri (`#[ignore]`); QA bahwa uninstaller `NEXORA.iss` menghapus semua metode.
- [ ] **Hide taskbar on boot**: sembunyikan taskbar per monitor (`platform/shell/taskbar.rs`), dipulihkan `nexora-guard` (REC-01).
- [ ] **Display font**: rendering teks memakai font pilihan dan skala font global.
- [ ] **STG gaps**: efek untuk icon di Finder, blur ikon latar, native preview, hidden mode, keep border, desktop icons 3 keadaan, dan kecepatan animasi.
- [ ] **WAN**: efek animasi jendela di `app/fx.rs` dan `app/motion.rs`.
- [ ] **SCR**: jendela overlay click-through atau gamma ramp; gamma wajib punya jalur restore.

## 7. Hardening (M7) dan lapisan tes
- [ ] **REC-01**: jalur restore di `nexora-guard` untuk taskbar per display, work area, parked windows, cursor, dan gamma; uji manual saat exit normal, panic, Ctrl-C, dan kill.
- [ ] **REC-02**: paksa tiap fitur gagal start, dan pastikan fitur lain tetap jalan.
- [ ] **QA-01**: satu `docs/qa/<id>.md` untuk setiap perilaku khusus Windows.
- [ ] **QA-02**: tes upgrade dengan `settings.cfg` 0.4.0 asli.
- [ ] **QA-03**: footprint tiap milestone.
- [ ] **L3, L4, L5, L7** di rencana tes: tes platform (`#[ignore]`), smoke `--settings-page`, QA manual, dan footprint.