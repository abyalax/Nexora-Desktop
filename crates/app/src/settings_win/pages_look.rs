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
    vec![
      (
        None,
        vec![RowDef::Seg(
          prefs::tr("Skala Deck, Visor, Menu Bar & Quick Panel"),
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
          RowDef::Seg(prefs::tr("Material Visor"), mat_opts(false), mat_idx(s.notch_material)),
          RowDef::Slider(prefs::tr("Tingkat blur"), s.blur, 0),
          RowDef::Slider(prefs::tr("Transparansi"), s.transparency, 1),
          RowDef::Toggle(prefs::tr("Refraksi tepi kaca (Deck)"), s.refraction, Action::Refraction),
        ],
      ),
      (
        Some(prefs::tr("Kustomisasi")),
        vec![
          RowDef::Toggle(
            prefs::tr("Blur Dock kustom"),
            s.extra.custom_dock_blur,
            Action::CustomDockBlur,
          ),
          RowDef::Toggle(
            prefs::tr("Refleksi yang disematkan"),
            s.extra.pinned_reflections,
            Action::PinnedReflections,
          ),
          RowDef::Toggle(
            prefs::tr("Label ikon kustom"),
            s.extra.custom_icon_labels,
            Action::CustomIconLabels,
          ),
          RowDef::Toggle(
            prefs::tr("Menu konteks kustom"),
            s.extra.custom_context_menus,
            Action::CustomContextMenus,
          ),
          RowDef::Toggle(
            prefs::tr("Grid folder kustom"),
            s.extra.custom_folder_grid,
            Action::CustomFolderGrid,
          ),
          RowDef::Toggle(
            prefs::tr("Menu Nexora kustom"),
            s.extra.custom_nexora_menus,
            Action::CustomNexoraMenus,
          ),
          RowDef::Toggle(
            prefs::tr("Blur Menu Bar kustom"),
            s.extra.custom_finder_blur,
            Action::CustomFinderBlur,
          ),
          RowDef::Toggle(
            prefs::tr("Blur Launchpad kustom"),
            s.extra.launchpad_custom_blur,
            Action::LaunchpadCustomBlur,
          ),
          RowDef::Toggle(
            prefs::tr("Tampilkan Launchpad di atas jendela"),
            s.extra.launchpad_above_windows,
            Action::LaunchpadAboveWindows,
          ),
          RowDef::Toggle(
            prefs::tr("Launchpad blur latar belakang"),
            s.extra.launchpad_blur_background,
            Action::LaunchpadBlurBackground,
          ),
        ],
      ),
    ]
  }
}
