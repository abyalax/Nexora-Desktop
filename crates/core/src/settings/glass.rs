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
}
