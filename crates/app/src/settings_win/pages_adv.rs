use super::*;

impl Win {
  pub(super) fn page_5(&self) -> Rows {
    let s = prefs::get();
    vec![
      (
        None,
        vec![
          RowDef::Toggle(prefs::tr("Aktifkan Scene"), s.stage, Action::StageOn),
          RowDef::Toggle(
            prefs::tr("Ganti Alt+Tab dengan pengalih nexora"),
            s.switcher,
            Action::SwitcherOn,
          ),
          RowDef::Toggle(
            prefs::tr("Klik wallpaper menyimpan semua jendela"),
            s.stage_desktop_click,
            Action::StageFlag(1),
          ),
          RowDef::Toggle(
            prefs::tr("Judul jendela saat hover"),
            s.stage_titles,
            Action::StageFlag(3),
          ),
          RowDef::Toggle(
            prefs::tr("Jeda media app di latar"),
            s.stage_media,
            Action::StageFlag(2),
          ),
        ],
      ),
      (
        Some(prefs::tr("Daftar jendela")),
        vec![
          RowDef::Seg(
            prefs::tr("Posisi daftar jendela"),
            vec![
              (prefs::tr("Kiri"), Action::StageRight(false)),
              (prefs::tr("Kanan"), Action::StageRight(true)),
            ],
            if s.stage_right { 1 } else { 0 },
          ),
          RowDef::Seg(
            prefs::tr("Ikon desktop"),
            vec![
              (prefs::tr("Sembunyi"), Action::StageIcons(0)),
              (prefs::tr("Otomatis"), Action::StageIcons(1)),
              (prefs::tr("Tampil"), Action::StageIcons(2)),
            ],
            s.stage_icons as usize,
          ),
          RowDef::Slider(prefs::tr("Ukuran kartu"), s.stage_size, 8),
          RowDef::Slider(prefs::tr("Jarak antar kartu"), s.stage_gap, 9),
          RowDef::Slider(prefs::tr("Jumlah kartu"), s.stage_count, 10),
          RowDef::Slider(prefs::tr("Kecepatan animasi"), s.stage_speed, 11),
        ],
      ),
      (
        Some(prefs::tr("Stage Manager")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon Stage Manager"),
            s.extra.stage_icon,
            Action::StageIcon,
          ),
          RowDef::Toggle(
            prefs::tr("Blur ikon jendela latar"),
            s.extra.stage_blur_icons,
            Action::StageBlurIcons,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan preview jendela native"),
            s.extra.stage_native_preview,
            Action::StageNativePreview,
          ),
          RowDef::Toggle(
            prefs::tr("Pertahankan Batas Jendela"),
            s.extra.stage_keep_border,
            Action::StageKeepBorder,
          ),
          RowDef::Seg(
            prefs::tr("Mode Tampilan Tersembunyi Daftar Jendela"),
            vec![
              (prefs::tr("Default"), Action::StageHiddenMode(0)),
              (prefs::tr("Selalu tampilkan"), Action::StageHiddenMode(1)),
              (prefs::tr("Selalu sembunyi"), Action::StageHiddenMode(2)),
            ],
            s.extra.stage_hidden_mode as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Menu Bar")),
        vec![
          RowDef::Toggle(prefs::tr("Ikon menu"), s.extra.finder_menu_icon, Action::FinderMenuIcon),
          RowDef::Toggle(prefs::tr("Auto-Hide"), s.extra.finder_auto_hide, Action::FinderAutoHide),
          RowDef::Toggle(
            prefs::tr("Konfirmasi shutdown"),
            s.extra.finder_shutdown_confirmation,
            Action::FinderShutdownConfirm,
          ),
          RowDef::Toggle(
            prefs::tr("Jendela maksimal memengaruhi tampilan"),
            s.extra.finder_maximized_affects_appearance,
            Action::FinderMaximizedAffects,
          ),
          RowDef::Toggle(
            prefs::tr("Tutup maksimal di sudut"),
            s.extra.finder_close_maximized_on_corner,
            Action::FinderCloseMaximizedCorner,
          ),
          RowDef::Toggle(
            prefs::tr("Kunci dragging ikon tray"),
            s.extra.finder_lock_tray_dragging,
            Action::FinderLockTrayDrag,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan opsi jendela"),
            s.extra.finder_window_options,
            Action::FinderWindowOptions,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon tray berwarna"),
            s.extra.finder_colored_tray_icons,
            Action::FinderColoredTray,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan menu metode input"),
            s.extra.finder_input_method_menu,
            Action::FinderInputMethod,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon pencarian"),
            s.extra.finder_search_icon,
            Action::FinderSearchIcon,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon akun"),
            s.extra.finder_account_icon,
            Action::FinderAccountIcon,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon pemutaran media"),
            s.extra.finder_media_playback_icon,
            Action::FinderMediaPlayback,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan menu lirik"),
            s.extra.finder_lyrics_menu,
            Action::FinderLyricsMenu,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan Lirik Pemain Warna"),
            s.extra.finder_color_player_lyrics,
            Action::FinderColorLyrics,
          ),
        ],
      ),
      (
        Some(prefs::tr("Notifikasi")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan notifikasi desktop"),
            s.extra.finder_show_notifications,
            Action::FinderShowNotif,
          ),
          RowDef::Toggle(
            prefs::tr("Suara notifikasi"),
            s.extra.finder_notification_sounds,
            Action::FinderNotifSounds,
          ),
          RowDef::Slider(
            prefs::tr("Durasi tampilan notifikasi"),
            s.extra.finder_notification_seconds as u8,
            26,
          ),
        ],
      ),
      (
        Some(prefs::tr("Waktu & Tanggal")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan waktu"),
            s.extra.finder_show_time,
            Action::FinderShowTime,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan tanggal"),
            s.extra.finder_show_date,
            Action::FinderShowDate,
          ),
          RowDef::Seg(
            prefs::tr("Format tanggal"),
            vec![
              (prefs::tr("Pendek"), Action::FinderDateFormat(0)),
              (prefs::tr("Panjang"), Action::FinderDateFormat(1)),
              (prefs::tr("ISO"), Action::FinderDateFormat(2)),
            ],
            s.extra.finder_date_format as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Audio")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan kontrol volume"),
            s.extra.finder_show_volume,
            Action::FinderShowVolume,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan pemilih output audio"),
            s.extra.finder_show_audio_output,
            Action::FinderShowAudioOutput,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan kontrol pemutaran media"),
            s.extra.finder_show_media_controls,
            Action::FinderShowMediaControls,
          ),
        ],
      ),
      (
        Some(prefs::tr("Tampilan")),
        vec![RowDef::Toggle(
          prefs::tr("Tampilkan kontrol kecerahan tampilan"),
          s.extra.finder_show_brightness,
          Action::FinderShowBrightness,
        )],
      ),
      (
        Some(prefs::tr("Status Hardware")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan status Wi-Fi"),
            s.extra.finder_show_wifi,
            Action::FinderShowWifi,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan status Bluetooth"),
            s.extra.finder_show_bluetooth,
            Action::FinderShowBluetooth,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon status perangkat"),
            s.extra.finder_show_hardware_icons,
            Action::FinderShowHardware,
          ),
        ],
      ),
      (
        Some(prefs::tr("Baterai")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan Menu Status Baterai"),
            s.extra.finder_show_battery_menu,
            Action::FinderShowBattery,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan waktu baterai tersisa"),
            s.extra.finder_remaining_battery_time,
            Action::FinderRemainingBattery,
          ),
          RowDef::Toggle(
            prefs::tr("Otomatis ganti mode daya"),
            s.extra.finder_auto_power_modes,
            Action::FinderAutoPowerModes,
          ),
          RowDef::Seg(
            prefs::tr("Mode daya saat dicolokkan"),
            vec![
              (prefs::tr("Seimbang"), Action::FinderPowerPlugged(0)),
              (prefs::tr("Performa terbaik"), Action::FinderPowerPlugged(1)),
              (prefs::tr("Efisiensi daya terbaik"), Action::FinderPowerPlugged(2)),
            ],
            s.extra.finder_power_mode_plugged as usize,
          ),
          RowDef::Seg(
            prefs::tr("Mode daya saat menggunakan baterai"),
            vec![
              (prefs::tr("Seimbang"), Action::FinderPowerBattery(0)),
              (prefs::tr("Performa terbaik"), Action::FinderPowerBattery(1)),
              (prefs::tr("Efisiensi daya terbaik"), Action::FinderPowerBattery(2)),
            ],
            s.extra.finder_power_mode_battery as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Tray")),
        vec![
          RowDef::Toggle(
            prefs::tr("Selalu tampilkan semua ikon tray di Menu Bar"),
            s.extra.tray_all_finder,
            Action::TrayAllFinder,
          ),
          RowDef::Toggle(
            prefs::tr("Selalu tampilkan semua ikon tray di Dock"),
            s.extra.tray_all_dock,
            Action::TrayAllDock,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan ikon tray sistem sendiri di Menu Bar"),
            s.extra.tray_system_finder,
            Action::TraySystemFinder,
          ),
          RowDef::Slider(
            prefs::tr("Waktu tampilan ikon tray baru"),
            s.extra.new_tray_icon_seconds as u8,
            27,
          ),
        ],
      ),
    ]
  }
}
