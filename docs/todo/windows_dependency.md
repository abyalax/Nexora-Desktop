This is TODO based on ../TODO.md, list ini khusus untuk task yang windows platform depend

## Progress note — 2026-11-27

This list separates implementation from validation. Several older rows described code as absent even though it is now in
the tree; those implementation sub-items are checked below or in `TODO.md`. Unchecked Windows 11 behavior, installer,
recovery, interactive-session, and footprint checks remain open until their QA procedures have actual results. This
checkout can compile the Windows workspace (`cargo check --workspace` passed after the current platform changes), but that
does not substitute for the manual checks.

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
- [x] **D-3.2 implementation**: app rejects drag start while locked (`dock/input.rs`); interactive QA remains open.
- [x] **D-6.2/6.4 implementation/decision**: preview reads size and delay live; "native preview" means a DWM thumbnail for Dock and STG-06.
- [ ] **D-7.5/7.7/7.8**: minimize biasa saat efek mati (QA), konten jendela live (`platform/wins/thumbnail.rs`), dan semantik "minimize into icon" (QA).
- [ ] **D-8.2/8.5/8.6**: enumerasi shortcut Start Menu dan Desktop, Launchpad per monitor, serta di atas jendela vs sembunyikan jendela dan blur live.
- [ ] **D-10.1/10.5/10.6/10.7/10.8**: pindahkan aturan visibilitas dari `app/dock/visibility.rs` ke core (butuh build untuk memastikan perilaku tidak berubah), hitung overlap (`wins/overlap.rs`), efek hide Move/Fade, reserve work area (QA: tidak ada sisa setelah ganti mode), dan ganti mode saat runtime.
- [ ] **D-11.5…11.8**: penempatan per sisi layar, render vertikal, anchor tooltip/preview/popup yang berbalik per sisi, dan target minimize dari posisi baru.
- [x] **D-12.2 implementation**: `platform/monitors.rs` enumerates monitor device ids, bounds, work areas, and primary state.
- [ ] **D-12.3/12.5**: rebuild on hotplug (QA) and expose detected monitors in the display selector.
- [ ] **D-13.4**: jendela hit-test tak terlihat (layered, no-activate, topmost).
- [ ] **D-14.3/14.4**: plate dan efek cahaya ikon.
- [ ] **D-15.3**: renderer refleksi, dengan cek footprint.
- [x] **D-16.2**: aksi display sleep, Start Menu, and Widgets are wired in `hotcorners.rs` (`platform::sys::display_off`, key actions).
- [ ] **D-17.2/17.3**: daftar isi folder, popup grid, double-click ke Explorer.
- [ ] **Wiring modul core M4** (core + tes sudah ada, butuh build Windows + QA di `docs/qa/`):
  - `dock/edge.rs` → D-11.5…11.8 (penempatan, render vertikal, anchor popup via `popup_side`, target minimize).
  - `dock/activation.rs` → D-13.4 (jendela hit-test memakai `hit_rect`, jam dari host ke `step`).
  - `dock/overlap.rs` → D-10.5 (`wins/overlap.rs` mengisi `Win` untuk `overlapped(scope, ...)`).
  - `dock/curve.rs` → D-14.3/14.4 (render plate/glow memakai `scale_at` dan `glow_at`).
  - `dock/reflection.rs` → D-15.3 (renderer refleksi memakai `derive` dan `opacity_at`).
  - `dock/folder.rs` → D-17.2/17.3 (popup grid memakai `grid` dan `cell_of`).
  - `launchpad/sources.rs` → D-8.2/8.5 (enumerasi Start Menu/Desktop menjadi `Entry`, `visible`, `grid`).

## 5. Finder / Menu Bar (M5)
Core M5 sudah selesai dan diuji di Linux (`cargo test -p nexora-core`); semua item di bawah belum pernah dibangun atau dijalankan di Windows.
- [ ] **F-4.5**: app memanggil `clock::format_clock` dan `items::build_with` (dengan `Show::from_settings` dan `Available`); saat ini app masih memakai `format` dan `build`.
- [ ] **F-8.3/8.4**: deteksi jendela maksimal per monitor (input `maximized` dan `overlapped`), reveal di tepi atas (`edge_armed`), dan pelepasan work area saat tersembunyi (QA, aturan di `appearance::reserves_work_area`).
- [ ] **F-9.2**: dialog konfirmasi shutdown bertema (Enter/Esc lewat `actions::dialog_result`), QA.
- [ ] **F-10.2**: kirim `WM_CLOSE` ke jendela foreground yang maksimal setelah `actions::can_close` (QA).
- [ ] **F-11.0** (spike): cara enumerasi dan hosting ikon tray; ini memblokir F-11.5…7. Riset meja ada di addendum ADR-0007, belum terverifikasi di Windows.
- [ ] **F-11.5…7**: enumerasi dan event perubahan tray, render, forward klik, drag reorder, plus row TRY-01…04, FLB-06, FLB-08.
- [ ] **F-12.0** (spike): API Windows untuk membaca notifikasi di app tanpa package identity; ini memblokir F-12.2…4 dan I-13. Riset meja ada di addendum ADR-0007, belum terverifikasi.
- [ ] **F-12.2/12.3**: listener notifikasi di platform dan menu Finder.
- [ ] **F-13.2/F-14**: menu audio (`platform/audio.rs`) dan brightness (`platform/brightness.rs`), plus row FAU-01…03 dan FDI-01.
- [ ] **F-15.2**: counter CPU/memori dari platform dan render item; ukur bahwa tidak ada polling lebih cepat dari 1 detik saat idle (QA-03).
- [ ] **F-16.4**: ganti power mode/plan dan event AC/baterai (QA).
- [ ] **F-17.2/F-18.2**: render media di bar dan row FLB-12…14, daftar layout keyboard, nama dan avatar akun.
- [ ] **Wiring modul core M5** (core + tes sudah ada, butuh build Windows + QA di `docs/qa/`):
  - `finder/items.rs` → `Show::from_settings` dan `Available` diisi dari platform (Bluetooth, volume, brightness, hardware, IME, akun, notifikasi, jumlah ikon tray, lebar media dari `media::budget`); id baru `BT`, `NOTIF`, `VOLUME`, `BRIGHT`, `HW`, `IME`, `ACCT`, `MEDIA`, `TRAY_MORE`, `TRAY0+i` perlu dihubungkan ke hit-test, menu, dan render.
  - `finder/clock.rs` → `format_clock` dengan `ClockStyle` dari `finder_show_time/date`, `clock_24h`, dan `finder_date_format`.
  - `finder/appearance.rs` → F-8.3/8.4 (`Signals` diisi host, `transition` menjalankan animasi hide/show, `bar_look(.., maximized)` menggantikan `settings::bar_look` untuk bar).
  - `finder/actions.rs` → F-9.2 (`needs_confirm` sebelum Restart/Shut Down) dan F-10.2 (`corner_rect`, `can_close`, `own_pid` dari host).
  - `finder/tray.rs` → F-11.5…7 (platform memanggil `upsert`/`remove`, UI memanggil `split` dan `move_to`, timer dari `next_expiry_ms`, warna dari `render`).
  - `finder/notifications.rs` → F-12.2/12.3 (listener memanggil `push`, menu membaca `live`, timer dari `next_expiry_ms`).
  - `finder/audio.rs` → F-13.2 (`Audio` dan `Outputs` disinkronkan dengan `platform/audio.rs`).
  - `finder/hardware.rs` → F-15.2 (`Sampler::due` dipanggil dari timer host, `reading` dari counter platform).
  - `finder/battery.rs` → F-16.4 (`AutoPlan::observe` dipanggil dari event AC/baterai, `remaining_text` dari estimasi Windows; estimasi tidak diketahui menjadi `None`).
  - `finder/media.rs` → F-17.2 (`budget`, `label`, `fit`, `fg`; teks lirik dari `Lyrics::current`, warna dari `tint::from_art`).

## 6. Halaman bersama (M6)
[ ] **Start on boot**: Registry and Scheduled Task methods are implemented in `platform/startup.rs`; app applies the selected method in `mainwin::apply_autostart`, and `installer/NEXORA.iss` removes both entries at uninstall. An ignored create/delete test exists; run it and perform manual installer QA for both methods. Core: `Settings::set_start_method`.
[ ] **Hide taskbar on boot**: sembunyikan/pulihkan taskbar per monitor di `platform/shell/taskbar.rs`, dipulihkan `nexora-guard` (REC-01). Core siap: `window_management/taskbar.rs` (`to_hide` untuk monitor yang disembunyikan, `to_restore` saat pilihan berubah).
[ ] **Display font**: pasang font Inter dari `assets/fonts`, cek font terpasang di platform, lalu rendering teks memakai `shared::font::resolve` dan `text_scale` (live). Core siap.
[ ] **STG gaps**: efek di `app/stage.rs` memakai `window_management/stage_options.rs`: icon di Finder, blur ikon latar, native preview, hidden mode (`strip_visible`), keep border, desktop icons 3 keadaan (`desktop_icons_visible`), kecepatan animasi, dan WAN-06 (`left_out_of_stage`). Asumsi yang perlu dikonfirmasi maintainer: arti "Default/Always show/Always hide" pada `HiddenMode`.
[ ] **WAN**: terapkan `window_management/animation.rs` (`frame`, `duration`, `should_animate`) di `app/fx.rs` dan `app/motion.rs`.
[ ] **SCR**: jendela overlay click-through (`dim_alpha`, `night_light`) atau gamma ramp (`gamma_gain`); target monitor lewat `screen::targets`; gamma wajib punya jalur restore di `nexora-guard`.

## 7. Hardening (M7) dan lapisan tes
Core M7 selesai dan diuji di Linux (`cargo test -p nexora-core`): `shared/features.rs` (REC-02), `tests/upgrade_0_4_0.rs` (QA-02), `tests/i18n_completeness.rs` (QA-04). Item di bawah belum pernah dibangun atau dijalankan di Windows.

### REC-01 Jalur restore di `nexora-guard` (`docs/qa/rec-01-recovery.md`)
- [x] taskbar, jendela terparkir, dan kursor sudah dipulihkan `guard` (ada di `crates/guard/src/main.rs`).
- [ ] taskbar **per display**: tulis ke marker di `platform/shell/taskbar.rs` dan pulihkan di `guard` (sama dengan "Hide taskbar on boot" di §6).
- [ ] reserve work area (Dock/Finder app bar): catat di marker, lepaskan di `guard`.
- [ ] gamma ramp: wajib punya jalur restore di `guard` sebelum SCR memakainya (§6).
- [ ] jalankan matriks 6 permukaan × 5 jalur keluar (normal, panic, Ctrl-C, kill, kill + guard).
- [ ] opsional: flag CLI untuk memicu panic (belum ada; rilis memakai `panic = "abort"`).

### REC-02 Fitur gagal start tidak menjatuhkan fitur lain (`docs/qa/rec-02-feature-failure.md`)
- [x] core: `shared/features.rs`, 6 tes (tiap dari 15 fitur dipaksa gagal, sisanya tetap jalan).
- [ ] `app/bootstrap.rs` sudah memakai `Startup` dan `NEXORA_FAIL_FEATURE` (kode ditulis, belum dikompilasi di Windows; cek `cargo clippy` untuk `redundant_closure`).
- [ ] jalankan tabel 16 baris di QA (15 fitur + `all`).

### QA-01 Satu `docs/qa/<id>.md` per perilaku khusus Windows
- [x] indeks dan prosedur M4/M5: `docs/qa/README.md` (semua berkas ditulis; hasil uji Windows 11 masih kosong).
- [ ] tulis tiap berkas bersama perilakunya, lalu isi hasilnya di Windows 11.

### QA-02 Tes upgrade
- [x] otomatis: `core/tests/upgrade_0_4_0.rs`.
- [ ] manual dengan `settings.cfg` 0.4.0 asli dan installer `NEXORA.iss` (`docs/qa/qa-02-upgrade.md`).

### QA-03 Footprint
- [ ] jalankan `tools/bench/m1-verify.ps1`, isi `docs/bench.md` (berkas ini belum ada dan belum ada angka terukur); ulangi di akhir tiap milestone.
- [ ] tidak ada timer lebih cepat dari 1 detik saat idle (I-12.3, F-15.2).

### Lain-lain
- [ ] **ADR-0009**: migrasi edition 2024 (`cargo fix --edition`), baru bisa diverifikasi dengan build Windows.
- [ ] **QA-05**: audit baris ✅ di `feature_checkpoint.md` terhadap perilaku Windows (§8 sudah memuat status M7).
- [ ] **L3, L4, L5, L7** di rencana tes: tes platform (`#[ignore]`), smoke `--settings-page`, QA manual, dan footprint.

## 8. Paritas HTML (M8)
Tidak ada task M8 yang bergantung pada Windows. **HTM-07** (`scripts/schema_parity.mjs`, tes `design/tests/schema-parity.test.mjs`, job CI `design`) berjalan dengan Node di Linux dan Windows, jadi dikerjakan dan diverifikasi di sini, bukan di daftar ini.
