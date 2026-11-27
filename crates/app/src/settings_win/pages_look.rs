use super::*;

impl Win {
  pub(super) fn page_4(&self) -> Rows {
    let s = prefs::get();
    use nexora_core::settings::IconStyle;
    let theme_idx = match s.theme {
      Theme::Auto => 0,
      Theme::Light => 1,
      Theme::Dark => 2,
    };
    let mat_idx = |m: Material| Material::ALL.iter().position(|x| *x == m).unwrap_or(0);
    let mat_opts = |dock: bool| {
      vec![
        (prefs::tr("Blur"), Action::Material(dock, Material::Blur)),
        (prefs::tr("Crystal"), Action::Material(dock, Material::Glass)),
        (prefs::tr("Hitam buram"), Action::Material(dock, Material::Black)),
      ]
    };
    let ist = match s.icon_style {
      IconStyle::Default => 0,
      IconStyle::Dark => 1,
      IconStyle::Clear => 2,
      IconStyle::Tinted => 3,
    };
    let accent_idx = if s.accent == nexora_core::settings::ACCENT_MULTI {
      0
    } else {
      s.accent as usize + 1
    };
    let mut theme = vec![
      RowDef::Swatches(prefs::tr("Warna"), accent_idx),
      RowDef::Thumbs(
        prefs::tr("Gaya ikon & widget"),
        vec![
          (
            prefs::tr("Bawaan"),
            Thumb::Icon(IconStyle::Default),
            Action::IconStyle(0),
          ),
          (prefs::tr("Gelap"), Thumb::Icon(IconStyle::Dark), Action::IconStyle(1)),
          (prefs::tr("Bening"), Thumb::Icon(IconStyle::Clear), Action::IconStyle(2)),
          (
            prefs::tr("Berwarna"),
            Thumb::Icon(IconStyle::Tinted),
            Action::IconStyle(3),
          ),
        ],
        ist,
      ),
    ];
    if s.icon_style != IconStyle::Default {
      let v = match s.icon_variant {
        1 => 0,
        2 => 1,
        _ => 2,
      };
      theme.push(RowDef::Radios(
        vec![
          (prefs::tr("Terang"), Action::IconVariant(1)),
          (prefs::tr("Gelap"), Action::IconVariant(2)),
          (prefs::tr("Otomatis"), Action::IconVariant(0)),
        ],
        v,
      ));
    }
    if s.icon_style == IconStyle::Tinted {
      theme.push(RowDef::Seg(
        prefs::tr("Warna tint"),
        vec![
          (prefs::tr("Biru"), Action::IconTint(0)),
          (prefs::tr("Ungu"), Action::IconTint(1)),
          (prefs::tr("Merah Muda"), Action::IconTint(2)),
          (prefs::tr("Oranye"), Action::IconTint(3)),
          (prefs::tr("Hijau"), Action::IconTint(4)),
        ],
        s.icon_tint as usize,
      ));
    }
    theme.push(RowDef::Toggle(
      prefs::tr("Paket ikon (folder iconpack)"),
      s.icon_pack,
      Action::IconPack,
    ));
    let scale_idx = match s.ui_scale {
      0..=112 => 0,
      113..=137 => 1,
      _ => 2,
    };
    let e = s.extra;
    let mut custom: Vec<RowDef> = Vec::new();
    // Dependent sliders remain visible and dimmed while their custom override is off.
    let mut group = |label: &'static str, on: bool, act: Action, sliders: &[(&'static str, u16, u8)]| {
      custom.push(RowDef::Toggle(prefs::tr(label), on, act));
      for (name, v, id) in sliders {
        custom.push(RowDef::GatedSlider(
          prefs::tr(name),
          (*v).min(u8::MAX as u16) as u8,
          *id,
          on,
        ));
      }
    };
    group(
      "Blur Dock kustom",
      e.custom_dock_blur,
      Action::CustomDockBlur,
      &[
        ("Intensitas blur Dock", e.dock_blur, 28),
        ("Transparansi Dock", e.dock_transparency, 29),
      ],
    );
    group(
      "Refleksi yang disematkan",
      e.pinned_reflections,
      Action::PinnedReflections,
      &[
        ("Blur refleksi", e.reflection_blur, 30),
        ("Transparansi refleksi", e.reflection_transparency, 31),
        ("Jarak refleksi ikon", e.reflection_distance, 32),
        ("Jarak gradasi refleksi", e.reflection_gradient_distance, 33),
      ],
    );
    group(
      "Label ikon kustom",
      e.custom_icon_labels,
      Action::CustomIconLabels,
      &[
        ("Blur label", e.label_blur, 34),
        ("Transparansi label", e.label_transparency, 35),
        ("Transparansi border label", e.label_border_transparency, 36),
      ],
    );
    group(
      "Menu konteks kustom",
      e.custom_context_menus,
      Action::CustomContextMenus,
      &[
        ("Blur menu konteks", e.context_blur, 37),
        ("Transparansi menu konteks", e.context_transparency, 38),
        ("Transparansi border menu", e.context_border, 39),
      ],
    );
    group(
      "Grid folder kustom",
      e.custom_folder_grid,
      Action::CustomFolderGrid,
      &[
        ("Blur grid folder", e.folder_blur, 40),
        ("Transparansi grid folder", e.folder_transparency, 41),
        ("Transparansi border grid", e.folder_border, 42),
        ("Ukuran ikon grid folder", e.folder_icon_size, 43),
      ],
    );
    group(
      "Menu Nexora kustom",
      e.custom_nexora_menus,
      Action::CustomNexoraMenus,
      &[
        ("Blur menu Nexora", e.menu_blur, 44),
        ("Transparansi menu Nexora", e.menu_transparency, 45),
        ("Transparansi border menu Nexora", e.menu_border, 46),
      ],
    );
    group(
      "Blur Menu Bar kustom",
      e.custom_finder_blur,
      Action::CustomFinderBlur,
      &[
        ("Blur Menu Bar", e.finder_blur, 47),
        ("Transparansi Menu Bar", e.finder_transparency, 48),
      ],
    );
    group(
      "Blur Launchpad kustom",
      e.launchpad_custom_blur,
      Action::LaunchpadCustomBlur,
      &[("Blur latar Launchpad", e.launchpad_blur, 50)],
    );
    custom.push(RowDef::Slider(
      prefs::tr("Intensitas tint wallpaper"),
      e.wallpaper_tint as u8,
      49,
    ));
    custom.push(RowDef::Toggle(
      prefs::tr("Tampilkan Launchpad di atas jendela"),
      e.launchpad_above_windows,
      Action::LaunchpadAboveWindows,
    ));
    custom.push(RowDef::Toggle(
      prefs::tr("Launchpad blur latar belakang"),
      e.launchpad_blur_background,
      Action::LaunchpadBlurBackground,
    ));
    vec![
      (
        None,
        vec![RowDef::Seg(
          prefs::tr("Skala Dock, Menubar, Menu Bar & Quick Panel"),
          vec![
            ("100%", Action::UiScale(100)),
            ("125%", Action::UiScale(125)),
            ("150%", Action::UiScale(150)),
          ],
          scale_idx,
        )],
      ),
      (
        None,
        vec![RowDef::Thumbs(
          prefs::tr("Tampilan"),
          vec![
            (prefs::tr("Otomatis"), Thumb::Appear(0), Action::Theme(Theme::Auto)),
            (prefs::tr("Terang"), Thumb::Appear(1), Action::Theme(Theme::Light)),
            (prefs::tr("Gelap"), Thumb::Appear(2), Action::Theme(Theme::Dark)),
          ],
          theme_idx,
        )],
      ),
      (Some(prefs::tr("Tema")), theme),
      (
        Some(prefs::tr("Kaca")),
        vec![
          RowDef::Seg(prefs::tr("Material"), mat_opts(true), mat_idx(s.dock_material)),
          RowDef::Seg(
            prefs::tr("Material Menubar"),
            mat_opts(false),
            mat_idx(s.notch_material),
          ),
          RowDef::Slider(prefs::tr("Tingkat blur"), s.blur, 0),
          RowDef::Slider(prefs::tr("Transparansi"), s.transparency, 1),
          RowDef::Toggle(prefs::tr("Refraksi tepi kaca (Deck)"), s.refraction, Action::Refraction),
        ],
      ),
      (Some(prefs::tr("Kustomisasi")), custom),
    ]
  }
}
