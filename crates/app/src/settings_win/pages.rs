use super::*;

impl Win {
  pub(super) fn page(&self, page: u8) -> Rows {
    match page {
      0 => self.page_0(),
      1 => self.page_1(),
      2 => self.page_2(),
      3 => self.page_3(),
      4 => self.page_4(),
      5 => self.page_5(),
      6 => self.page_6(),
      8 => self.page_8(),
      _ => self.page_x(),
    }
  }
  pub(super) fn page_0(&self) -> Rows {
    let s = prefs::get();
    vec![
      (
        None,
        vec![RowDef::Seg(
          prefs::tr("Bahasa"),
          vec![
            ("Indonesia", Action::Lang(Lang::Id)),
            ("English", Action::Lang(Lang::En)),
          ],
          if s.lang == Lang::Id { 0 } else { 1 },
        )],
      ),
      (
        Some(prefs::tr("Sistem")),
        vec![
          RowDef::Toggle(prefs::tr("Jalankan saat login Windows"), s.autostart, Action::Autostart),
          RowDef::Toggle(prefs::tr("Sembunyikan taskbar Windows"), s.takeover, Action::Takeover),
          RowDef::Toggle(
            prefs::tr("Sembunyikan ikon desktop saat nexora berjalan"),
            s.hide_desktop_icons,
            Action::HideIcons,
          ),
          RowDef::Toggle(
            prefs::tr("Selalu di atas jendela layar penuh"),
            s.above_fullscreen,
            Action::AboveFull,
          ),
          RowDef::Toggle(
            prefs::tr("Pintasan cepat (Alt+` ganti jendela app)"),
            s.mac_shortcuts,
            Action::MacKeys,
          ),
        ],
      ),
      (
        Some(prefs::tr("Jendela")),
        vec![
          RowDef::Toggle(
            prefs::tr("Animasi jendela (minimize, tutup, buka)"),
            s.anim,
            Action::Anim,
          ),
          RowDef::Toggle(
            prefs::tr("Bayangan halus di bawah jendela aktif"),
            s.win_shadow,
            Action::WinShadow,
          ),
        ],
      ),
      (
        Some(prefs::tr("Sistem")),
        vec![
          RowDef::Seg(
            prefs::tr("Metode start saat boot"),
            vec![
              (prefs::tr("Jangan start saat boot"), Action::StartMethod(0)),
              (prefs::tr("Registry"), Action::StartMethod(1)),
              (prefs::tr("Scheduled Task"), Action::StartMethod(2)),
            ],
            s.extra.start_method as usize,
          ),
          RowDef::Seg(
            prefs::tr("Sembunyikan taskbar saat boot"),
            vec![
              (prefs::tr("Tidak ada"), Action::TaskbarScope(0)),
              (prefs::tr("Semua layar"), Action::TaskbarScope(1)),
              (prefs::tr("Layar Dock"), Action::TaskbarScope(2)),
            ],
            s.extra.taskbar_scope as usize,
          ),
          RowDef::Seg(
            prefs::tr("Font tampilan"),
            vec![
              (prefs::tr("Segoe UI"), Action::DisplayFont(0)),
              (prefs::tr("Arial"), Action::DisplayFont(1)),
              (prefs::tr("Inter"), Action::DisplayFont(2)),
              (prefs::tr("Roboto"), Action::DisplayFont(3)),
              (prefs::tr("System UI"), Action::DisplayFont(4)),
            ],
            s.extra.display_font as usize,
          ),
          RowDef::Slider(
            prefs::tr("Skala font global"),
            ((s.extra.font_scale as f32 - 75.0) / 75.0 * 100.0).round() as u8,
            12,
          ),
        ],
      ),
      (
        Some(prefs::tr("Animasi Jendela")),
        vec![
          RowDef::Toggle(
            prefs::tr("Hapus jendela program yang dikecualikan"),
            s.extra.minimize_excluded_windows,
            Action::MinimizeExcludedWindows,
          ),
          RowDef::Seg(
            prefs::tr("Efek animasi"),
            vec![
              (prefs::tr("Scale"), Action::WindowAnimEffect(0)),
              (prefs::tr("Fade"), Action::WindowAnimEffect(1)),
              (prefs::tr("Slide"), Action::WindowAnimEffect(2)),
              (prefs::tr("Zoom"), Action::WindowAnimEffect(3)),
            ],
            s.extra.window_anim_effect as usize,
          ),
          RowDef::Seg(
            prefs::tr("Kecepatan animasi"),
            vec![
              (prefs::tr("0.5x"), Action::WindowAnimSpeed(0)),
              (prefs::tr("0.75x"), Action::WindowAnimSpeed(1)),
              (prefs::tr("1.0x"), Action::WindowAnimSpeed(2)),
              (prefs::tr("1.25x"), Action::WindowAnimSpeed(3)),
              (prefs::tr("1.5x"), Action::WindowAnimSpeed(4)),
            ],
            s.extra.window_anim_speed as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Layar")),
        vec![
          RowDef::Slider(prefs::tr("Kecerahan layar"), s.extra.screen_brightness as u8, 12),
          RowDef::Toggle(prefs::tr("Night light"), s.extra.night_light, Action::NightLight),
          RowDef::Slider(
            prefs::tr("Intensitas night light"),
            s.extra.night_light_intensity as u8,
            12,
          ),
          RowDef::Seg(
            prefs::tr("Target layar"),
            vec![
              (prefs::tr("Semua layar"), Action::ScreenTarget(0)),
              (prefs::tr("Layar utama"), Action::ScreenTarget(1)),
              (prefs::tr("Layar aktif"), Action::ScreenTarget(2)),
            ],
            s.extra.screen_target as usize,
          ),
        ],
      ),
      (
        Some(prefs::tr("Program")),
        vec![
          RowDef::Text(
            prefs::tr("Program yang dikecualikan"),
            {
              let lists = prefs::lists();
              lists.to_text(ListKind::ExcludedPrograms)
            },
            "pisahkan dengan titik koma".to_string(),
            8,
          ),
          RowDef::Text(
            prefs::tr("Program blacklist"),
            {
              let lists = prefs::lists();
              lists.to_text(ListKind::ProgramBlacklist)
            },
            "pisahkan dengan titik koma".to_string(),
            9,
          ),
          RowDef::Text(
            prefs::tr("Blacklist animasi minimize"),
            {
              let lists = prefs::lists();
              lists.to_text(ListKind::MinimizeBlacklist)
            },
            "pisahkan dengan titik koma".to_string(),
            10,
          ),
          RowDef::Text(
            prefs::tr("Program Start Menu yang dihapus"),
            {
              let lists = prefs::lists();
              lists.to_text(ListKind::DeletedStartMenuPrograms)
            },
            "pisahkan dengan titik koma".to_string(),
            11,
          ),
        ],
      ),
      (
        Some(prefs::tr("Tentang")),
        vec![RowDef::Pick(
          prefs::tr("Dibuat oleh kreativa.dev"),
          prefs::tr("Belikan aku kopi").to_string(),
          Action::Donate,
        )],
      ),
    ]
  }
  pub(super) fn page_2(&self) -> Rows {
    let s = prefs::get();
    vec![
      (
        None,
        vec![
          RowDef::Toggle(prefs::tr("Aktifkan Menubar"), s.notch_enabled, Action::NotchOn),
          RowDef::Seg(
            prefs::tr("Bentuk"),
            vec![
              (prefs::tr("Menubar"), Action::NotchShape(false)),
              (prefs::tr("Capsule"), Action::NotchShape(true)),
            ],
            if s.notch_island { 1 } else { 0 },
          ),
          RowDef::Seg(prefs::tr("Perilaku"), behaviors(false), beh_idx(s.notch)),
          RowDef::Slider(prefs::tr("Kebulatan sudut"), s.notch_round, 3),
          RowDef::Toggle(
            prefs::tr("Buka otomatis saat kursor di Menubar"),
            s.notch_hover_open,
            Action::HoverOpen,
          ),
        ],
      ),
      (Some(prefs::tr("Saat Menubar diam")), {
        let name = if s.greet_name.is_empty() {
          user_name()
        } else {
          s.greet_name.as_str().to_string()
        };
        let hour = nexora_platform::time::local_now().hour;
        let auto = nexora_core::dynamic_island::glance::greeting_text("", &name, hour, s.lang);
        vec![
          RowDef::Seg(
            prefs::tr("Tampilkan"),
            vec![
              (prefs::tr("Sapaan & info"), Action::NotchIdle(0)),
              (prefs::tr("Musik & lirik"), Action::NotchIdle(2)),
              (prefs::tr("Kosong"), Action::NotchIdle(1)),
            ],
            match s.notch_idle {
              0 => 0,
              2 => 1,
              _ => 2,
            },
          ),
          RowDef::Text(
            prefs::tr("Nama panggilan"),
            s.greet_name.as_str().to_string(),
            user_name(),
            0,
          ),
          RowDef::Text(prefs::tr("Teks sapaan"), s.greet_text.as_str().to_string(), auto, 1),
        ]
      }),
      (
        Some(prefs::tr("Lirik")),
        vec![
          RowDef::Toggle(
            prefs::tr("Tampilkan lirik di Menubar"),
            s.lyrics_compact,
            Action::LyricsCompact,
          ),
          RowDef::Toggle(
            prefs::tr("Cari lirik online (LRCLIB)"),
            s.lyrics_online,
            Action::LyricsOnline,
          ),
          RowDef::Pick(
            prefs::tr("Folder lirik (.lrc)"),
            prefs::tr("Buka").to_string(),
            Action::LyricsDir,
          ),
        ],
      ),
      (
        Some(prefs::tr("Aktivitas")),
        vec![
          RowDef::Toggle(prefs::tr("Media dan lirik"), s.extra.notch_media, Action::NotchMedia),
          RowDef::Toggle(
            prefs::tr("Tampilkan unduhan (Store, Steam, peramban)"),
            s.notch_downloads,
            Action::NotchDownloads,
          ),
          RowDef::Toggle(prefs::tr("Rak file"), s.extra.notch_shelf, Action::NotchShelf),
          RowDef::Toggle(
            prefs::tr("Kirim ke HP lewat QR (Wi-Fi yang sama)"),
            s.notch_send,
            Action::NotchSend,
          ),
          RowDef::Toggle(
            prefs::tr("Timer dan strip kalender"),
            s.extra.notch_timer,
            Action::NotchTimer,
          ),
        ],
      ),
    ]
  }
  pub(super) fn page_3(&self) -> Rows {
    let s = prefs::get();
    let cur = if s.menubar_material {
      3
    } else if !s.menubar_glass {
      0
    } else {
      match s.menubar_style {
        nexora_core::settings::BarStyle::Light => 1,
        nexora_core::settings::BarStyle::Dark => 2,
      }
    };
    vec![
      (
        None,
        vec![
          RowDef::Toggle(prefs::tr("Aktifkan Menu Bar"), s.menubar, Action::MenuBarOn),
          RowDef::Seg(
            prefs::tr("Latar Menu Bar"),
            vec![
              (prefs::tr("Transparan"), Action::BarStyle(0)),
              (prefs::tr("Terang"), Action::BarStyle(1)),
              (prefs::tr("Gelap"), Action::BarStyle(2)),
              (prefs::tr("Material"), Action::BarStyle(3)),
            ],
            cur,
          ),
          RowDef::Toggle(prefs::tr("Jam 24 jam"), s.clock_24h, Action::Clock24),
          RowDef::Seg(
            prefs::tr("Persentase baterai"),
            vec![
              (prefs::tr("Di dalam"), Action::BatteryPct(1)),
              (prefs::tr("Di luar"), Action::BatteryPct(2)),
              (prefs::tr("Sembunyi"), Action::BatteryPct(0)),
            ],
            match s.battery_pct {
              1 => 0,
              2 => 1,
              _ => 2,
            },
          ),
        ],
      ),
      (Some(prefs::tr("Logo Menu Bar")), {
        let custom = crate::menubar::logo::path().is_some();
        let mut rows = vec![RowDef::Pick(
          prefs::tr("Logo Menu Bar"),
          prefs::tr(if custom { "Gambar sendiri" } else { "Lingkaran bawaan" }).to_string(),
          Action::MenuLogoPick,
        )];
        if custom {
          rows.push(RowDef::Pick(
            prefs::tr("Logo kustom"),
            prefs::tr("Kembalikan bawaan").to_string(),
            Action::MenuLogoReset,
          ));
        }
        rows
      }),
    ]
  }
  pub(super) fn page_6(&self) -> Rows {
    let s = prefs::get();
    let names = ["Kiri atas", "Kanan atas", "Kiri bawah", "Kanan bawah"];
    vec![(
      None,
      names
        .iter()
        .enumerate()
        .map(|(i, nm)| {
          RowDef::Pick(
            prefs::tr(nm),
            prefs::tr(hot_label(s.hot[i])).to_string(),
            Action::HotCycle(i),
          )
        })
        .collect(),
    )]
  }
  pub(super) fn page_8(&self) -> Rows {
    let s = prefs::get();
    vec![(
      None,
      vec![
        RowDef::Seg(
          prefs::tr("Material widget"),
          vec![
            (prefs::tr("Original"), Action::WidgetStyle(0)),
            (prefs::tr("Crystal"), Action::WidgetStyle(1)),
          ],
          s.widget_style.min(1) as usize,
        ),
        RowDef::Pick(
          prefs::tr("Widget desktop"),
          prefs::tr("Tambah widget").to_string(),
          Action::WidgetGallery,
        ),
      ],
    )]
  }
  pub(super) fn page_x(&self) -> Rows {
    let s = prefs::get();
    vec![(
      None,
      vec![RowDef::Toggle(
        prefs::tr("Riwayat Papan Klip di Quick Search"),
        s.clipboard_history,
        Action::ClipHistory,
      )],
    )]
  }
}

pub(super) fn behaviors(dock: bool) -> Vec<(&'static str, Action)> {
  [
    (prefs::tr("Tetap"), Behavior::Fixed),
    (prefs::tr("Menimpa"), Behavior::Overlay),
    (prefs::tr("Pintar"), Behavior::Smart),
    (prefs::tr("Sesuai permintaan"), Behavior::OnDemand),
  ]
  .into_iter()
  .filter(|(_, b)| !dock || Behavior::DOCK.contains(b))
  .map(|(t, b)| (t, if dock { Action::Dock(b) } else { Action::Notch(b) }))
  .collect()
}

pub(super) fn beh_idx(b: Behavior) -> usize {
  match b {
    Behavior::Fixed => 0,
    Behavior::Overlay => 1,
    Behavior::Smart => 2,
    // Only the Island offers this segment; a Dock file that says `ondemand` shows the last Dock option.
    Behavior::OnDemand => 3,
  }
}
