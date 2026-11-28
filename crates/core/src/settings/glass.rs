//! Per-surface glass overrides. A surface inherits the global values until its custom switch is enabled.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlassValues {
  pub blur: u8,
  pub transparency: u8,
  pub border: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SurfaceGlass {
  pub custom: bool,
  pub blur: u8,
  pub transparency: u8,
  pub border: u8,
}

impl SurfaceGlass {
  pub const fn values(self) -> GlassValues {
    GlassValues {
      blur: self.blur,
      transparency: self.transparency,
      border: self.border,
    }
  }

  pub const fn resolve(global: GlassValues, surface: SurfaceGlass) -> GlassValues {
    if surface.custom { surface.values() } else { global }
  }
}

/// A surface that has its own "use custom blur and transparency values" group on the Appearance page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
  Dock,
  Labels,
  ContextMenus,
  FolderGrid,
  NexoraMenus,
  Finder,
  Launchpad,
}

impl Surface {
  pub const ALL: [Surface; 7] = [
    Surface::Dock,
    Surface::Labels,
    Surface::ContextMenus,
    Surface::FolderGrid,
    Surface::NexoraMenus,
    Surface::Finder,
    Surface::Launchpad,
  ];

  /// `(custom switch, blur slider, transparency slider)` in the `Extra` registry. Launchpad has no transparency row:
  /// its transparency follows the global value.
  pub const fn keys(self) -> (&'static str, &'static str, Option<&'static str>) {
    match self {
      Surface::Dock => ("custom_dock_blur", "dock_blur", Some("dock_transparency")),
      Surface::Labels => ("custom_icon_labels", "label_blur", Some("label_transparency")),
      Surface::ContextMenus => ("custom_context_menus", "context_blur", Some("context_transparency")),
      Surface::FolderGrid => ("custom_folder_grid", "folder_blur", Some("folder_transparency")),
      Surface::NexoraMenus => ("custom_nexora_menus", "menu_blur", Some("menu_transparency")),
      Surface::Finder => ("custom_finder_blur", "finder_blur", Some("finder_transparency")),
      Surface::Launchpad => ("launchpad_custom_blur", "launchpad_blur", None),
    }
  }
}

/// A surface's own values, present only while its custom switch is on. `blur` is the slider value (0-50, a sigma-like
/// number); `transparency` is 0-100 where 100 is fully see-through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CustomGlass {
  pub blur: u16,
  pub transparency: Option<u16>,
}

impl super::Extra {
  /// `None` while the surface's custom switch is off (it then follows the global glass).
  pub fn custom_glass(&self, surface: Surface) -> Option<CustomGlass> {
    let (switch, blur, transparency) = surface.keys();
    if self.get(switch).unwrap_or(0) == 0 {
      return None;
    }
    let num = |key: &str| self.get(key).unwrap_or(0).clamp(0, 100) as u16;
    Some(CustomGlass {
      blur: num(blur).min(50),
      transparency: transparency.map(num),
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_off_uses_global_values() {
    let global = GlassValues {
      blur: 64,
      transparency: 72,
      border: 18,
    };
    let surface = SurfaceGlass {
      custom: false,
      blur: 8,
      transparency: 9,
      border: 10,
    };
    assert_eq!(SurfaceGlass::resolve(global, surface), global);
  }

  #[test]
  fn custom_on_uses_surface_values() {
    let surface = SurfaceGlass {
      custom: true,
      blur: 8,
      transparency: 9,
      border: 10,
    };
    assert_eq!(SurfaceGlass::resolve(GlassValues::default(), surface), surface.values());
  }

  #[test]
  fn every_surface_key_exists_in_the_registry() {
    let e = super::super::Extra::default();
    for surface in Surface::ALL {
      let (switch, blur, transparency) = surface.keys();
      for key in [Some(switch), Some(blur), transparency].into_iter().flatten() {
        assert!(e.get(key).is_some(), "{surface:?}: unknown key {key}");
      }
    }
  }

  #[test]
  fn custom_off_gives_nothing_even_when_the_sliders_moved() {
    let mut e = super::super::Extra::default();
    for surface in Surface::ALL {
      let (_, blur, transparency) = surface.keys();
      assert!(e.apply(blur, "30"));
      if let Some(t) = transparency {
        assert!(e.apply(t, "20"));
      }
      assert_eq!(e.custom_glass(surface), None, "{surface:?}");
    }
  }

  #[test]
  fn custom_on_returns_that_surfaces_own_values() {
    let mut e = super::super::Extra::default();
    assert!(e.apply("custom_icon_labels", "on"));
    assert!(e.apply("label_blur", "12"));
    assert!(e.apply("label_transparency", "40"));
    assert_eq!(
      e.custom_glass(Surface::Labels),
      Some(CustomGlass {
        blur: 12,
        transparency: Some(40)
      })
    );
    assert_eq!(
      e.custom_glass(Surface::ContextMenus),
      None,
      "another surface is untouched"
    );
  }

  #[test]
  fn blur_is_capped_at_50_and_launchpad_has_no_transparency() {
    let mut e = super::super::Extra::default();
    assert!(e.apply("launchpad_custom_blur", "on"));
    assert!(e.apply("launchpad_blur", "50"));
    assert_eq!(
      e.custom_glass(Surface::Launchpad),
      Some(CustomGlass {
        blur: 50,
        transparency: None
      })
    );
  }
}
