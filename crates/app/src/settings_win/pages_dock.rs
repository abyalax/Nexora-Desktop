use super::*;

impl Win {
  pub(super) fn page_1(&self) -> Rows {
    let s = prefs::get();
    vec![
      (
        None,
        vec![
          RowDef::Toggle(prefs::tr("Aktifkan Dock"), s.dock_enabled, Action::DockOn),
          RowDef::Seg(prefs::tr("Perilaku"), behaviors(true), beh_idx(s.dock).min(2)),
          RowDef::Seg(
            prefs::tr("Efek minimize"),
            vec![
              (prefs::tr("Genie"), Action::MinEffect(0)),
              (prefs::tr("Skala"), Action::MinEffect(1)),
              (prefs::tr("Suck"), Action::MinEffect(2)),
            ],
            s.min_effect as usize,
          ),
          RowDef::Toggle(prefs::tr("Ikon seragam (squircle)"), s.dock_plates, Action::DockPlates),
          RowDef::Toggle(prefs::tr("Tumpukan Unduhan"), s.downloads_stack, Action::DownloadsStack),
          RowDef::Toggle(prefs::tr("App terakhir dipakai"), s.recent_apps, Action::RecentApps),
        ],
      ),
      (
        Some(prefs::tr("Ukuran")),
        vec![
          RowDef::Slider(prefs::tr("Ukuran ikon"), s.dock_size, 4),
          RowDef::Slider(prefs::tr("Jarak antar ikon"), s.dock_gap, 5),
          RowDef::GatedSlider(
            prefs::tr("Zoom saat hover"),
            s.dock_zoom,
            6,
            nexora_core::settings::Dependencies::from_settings(&s.extra).dock_magnification,
          ),
          RowDef::Slider(prefs::tr("Jarak dari tepi bawah"), s.dock_margin, 7),
          RowDef::Slider(prefs::tr("Kebulatan sudut"), s.dock_round, 2),
        ],
      ),
      (
        Some(prefs::tr("Tampilan & Perilaku Dock")),
        vec![
          RowDef::Seg(
            prefs::tr("Efek ikon"),
            vec![
              (prefs::tr("Tidak ada"), Action::DockIconEffect(0)),
              (prefs::tr("Scaling"), Action::DockIconEffect(1)),
              (prefs::tr("Select"), Action::DockIconEffect(2)),
              (prefs::tr("Light"), Action::DockIconEffect(3)),
              (prefs::tr("Scaling Light"), Action::DockIconEffect(4)),
              (prefs::tr("Select Light"), Action::DockIconEffect(5)),
            ],
            s.extra.dock_icon_effect as usize,
          ),
          RowDef::Slider(
            prefs::tr("Intensitas cahaya ikon"),
            s.extra.dock_icon_light_intensity as u8,
            15,
          ),
          RowDef::Slider(
            prefs::tr("Intensitas cahaya latar ikon"),
            s.extra.dock_icon_background_light as u8,
            16,
          ),
          RowDef::Seg(
            prefs::tr("Posisi di layar"),
            vec![
              (prefs::tr("Kiri"), Action::DockPosition(0)),
              (prefs::tr("Bawah"), Action::DockPosition(1)),
              (prefs::tr("Kanan"), Action::DockPosition(2)),
              (prefs::tr("Atas"), Action::DockPosition(3)),
            ],
            s.extra.dock_position as usize,
          ),
          RowDef::Seg(
            prefs::tr("Tampilan di layar"),
            vec![
              (prefs::tr("Layar utama"), Action::DockDisplay(0)),
              (prefs::tr("Setiap layar terdeteksi"), Action::DockDisplay(1)),
            ],
            s.extra.dock_display as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Launchpad")),
        vec![
          RowDef::Slider(prefs::tr("Ukuran ikon"), s.extra.launchpad_icon_size as u8, 17),
          RowDef::Slider(prefs::tr("Jarak horizontal"), s.extra.launchpad_h_spacing as u8, 18),
          RowDef::Slider(prefs::tr("Jarak vertikal"), s.extra.launchpad_v_spacing as u8, 19),
          RowDef::Seg(
            prefs::tr("Sumber"),
            vec![
              (prefs::tr("Ditambahkan"), Action::LaunchpadSources(0)),
              (prefs::tr("Ditambahkan & Start Menu"), Action::LaunchpadSources(1)),
              (prefs::tr("Ditambahkan & Desktop"), Action::LaunchpadSources(2)),
              (prefs::tr("Semua"), Action::LaunchpadSources(3)),
            ],
            s.extra.launchpad_sources as usize,
          ),
          RowDef::Seg(
            prefs::tr("Tampilan"),
            vec![
              (prefs::tr("Layar utama"), Action::LaunchpadDisplay(0)),
              (prefs::tr("Semua layar"), Action::LaunchpadDisplay(1)),
              (prefs::tr("Layar 1"), Action::LaunchpadDisplay(2)),
              (prefs::tr("Layar 2"), Action::LaunchpadDisplay(3)),
            ],
            s.extra.launchpad_display as usize,
          ),
          RowDef::Toggle(
            prefs::tr("Ikuti Dock"),
            s.extra.launchpad_follow_dock,
            Action::LaunchpadFollowDock,
          ),
        ],
      ),
      (
        Some(prefs::tr("Lanjutan Dock")),
        vec![
          RowDef::Toggle(
            prefs::tr("Klik ganda folder"),
            s.extra.dock_dblclick_folder,
            Action::DockDblClickFolder,
          ),
          RowDef::Toggle(
            prefs::tr("Sembunyikan tombol folder terbuka"),
            s.extra.dock_hide_open_folder_button,
            Action::DockHideOpenFolder,
          ),
          RowDef::Toggle(
            prefs::tr("Kunci dragging"),
            s.extra.dock_lock_dragging,
            Action::DockLockDrag,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan label"),
            s.extra.dock_show_labels,
            Action::DockShowLabels,
          ),
          RowDef::Toggle(
            prefs::tr("Animasi peluncuran"),
            s.extra.dock_animate_launch,
            Action::DockAnimateLaunch,
          ),
          RowDef::Toggle(
            prefs::tr("Cegah penutupan"),
            s.extra.dock_prevent_cover,
            Action::DockPreventCover,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan indikator"),
            s.extra.dock_show_indicators,
            Action::DockShowIndicators,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan yang tidak disematkan"),
            s.extra.dock_show_unpinned,
            Action::DockShowUnpinned,
          ),
          RowDef::Toggle(
            prefs::tr("Pisahkan yang tidak disematkan"),
            s.extra.dock_separate_unpinned,
            Action::DockSeparateUnpinned,
          ),
          RowDef::Seg(
            prefs::tr("Jendela deteksi"),
            vec![
              (prefs::tr("Semua jendela program"), Action::DockDetection(0)),
              (prefs::tr("Jendela program saat ini"), Action::DockDetection(1)),
            ],
            s.extra.dock_detection as usize,
          ),
          RowDef::Seg(
            prefs::tr("Efek tersembunyi"),
            vec![
              (prefs::tr("Geser"), Action::DockHiddenEffect(0)),
              (prefs::tr("Memudar"), Action::DockHiddenEffect(1)),
            ],
            s.extra.dock_hidden_effect as usize,
          ),
          RowDef::Seg(
            prefs::tr("Pemicu aktivasi"),
            vec![
              (prefs::tr("Tepi layar"), Action::DockActivationTrigger(0)),
              (prefs::tr("Bar aktivasi"), Action::DockActivationTrigger(1)),
              (prefs::tr("Tepi layar & Bar aktivasi"), Action::DockActivationTrigger(2)),
            ],
            s.extra.dock_activation_trigger as usize,
          ),
          RowDef::Slider(prefs::tr("Lebar bar aktivasi"), s.extra.dock_act_bar_width as u8, 20),
          RowDef::Slider(prefs::tr("Tinggi bar aktivasi"), s.extra.dock_act_bar_height as u8, 21),
          RowDef::Slider(prefs::tr("Jarak bar aktivasi"), s.extra.dock_act_bar_distance as u8, 22),
          RowDef::WideSlider(
            prefs::tr("Delay aktivasi mouse"),
            s.extra.dock_activation_delay as f32,
            23,
            nexora_core::settings::NumericSlider::new(0.0, 1500.0, 10.0).unwrap_or_default(),
            " ms",
          ),
        ],
      ),
      (
        Some(prefs::tr("Efek Minimize")),
        vec![
          RowDef::Toggle(
            prefs::tr("Gunakan minimize Nexora"),
            s.extra.use_nexora_minimize,
            Action::UseNexoraMinimize,
          ),
          RowDef::Toggle(
            prefs::tr("Jangan animasikan minimize"),
            s.extra.disable_minimize_anim,
            Action::DisableMinimizeAnim,
          ),
          RowDef::Toggle(
            prefs::tr("Tahan Shift untuk perlambat"),
            s.extra.shift_slow_anim,
            Action::ShiftSlowAnim,
          ),
          RowDef::Toggle(
            prefs::tr("Minimize ke ikon"),
            s.extra.minimize_into_icon,
            Action::MinimizeIntoIcon,
          ),
          RowDef::Toggle(
            prefs::tr("Konten minimize dinamis"),
            s.extra.dynamic_minimize_content,
            Action::DynamicMinimizeContent,
          ),
          RowDef::Seg(
            prefs::tr("Posisi ikon diminimalkan"),
            vec![
              (prefs::tr("Di samping ikon"), Action::MinimizedIconPos(0)),
              (prefs::tr("Independen"), Action::MinimizedIconPos(1)),
              (prefs::tr("Pemisah"), Action::MinimizedIconPos(2)),
            ],
            s.extra.minimized_icon_position as usize,
          ),
          RowDef::Seg(
            prefs::tr("Kecepatan animasi minimize"),
            vec![
              (prefs::tr("0.5x"), Action::MinimizeAnimSpeed(0)),
              (prefs::tr("0.75x"), Action::MinimizeAnimSpeed(1)),
              (prefs::tr("1.0x"), Action::MinimizeAnimSpeed(2)),
              (prefs::tr("1.25x"), Action::MinimizeAnimSpeed(3)),
              (prefs::tr("1.5x"), Action::MinimizeAnimSpeed(4)),
            ],
            s.extra.minimize_anim_speed as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Preview Jendela")),
        vec![
          RowDef::Toggle(
            prefs::tr("Aktifkan preview jendela"),
            s.extra.window_previews,
            Action::WindowPreviews,
          ),
          RowDef::Slider(
            prefs::tr("Ukuran preview"),
            ((s.extra.preview_size - 120) / 4) as u8,
            24,
          ),
          RowDef::WideSlider(
            prefs::tr("Delay preview"),
            s.extra.preview_delay as f32,
            25,
            nexora_core::settings::NumericSlider::new(0.0, 1500.0, 10.0).unwrap_or_default(),
            " ms",
          ),
          RowDef::Toggle(
            prefs::tr("Nama aplikasi"),
            s.extra.preview_app_name,
            Action::PreviewAppName,
          ),
          RowDef::Toggle(
            prefs::tr("Tombol kontrol jendela"),
            s.extra.preview_controls,
            Action::PreviewControls,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan preview native"),
            s.extra.preview_native,
            Action::PreviewNative,
          ),
        ],
      ),
    ]
  }
}
