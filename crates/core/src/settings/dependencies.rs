//! Pure enablement rules shared by settings presentation and effects.

use super::Extra;
use crate::dock::behavior::IconEffect;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dependencies {
  pub custom_dock_glass_values: bool,
  pub dock_magnification: bool,
}

impl Dependencies {
  pub fn from_settings(extra: &Extra) -> Self {
    Self {
      custom_dock_glass_values: extra.custom_dock_blur,
      dock_magnification: IconEffect::ALL
        .get(extra.dock_icon_effect as usize)
        .is_some_and(|effect| effect.magnifies()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn custom_glass_values_follow_the_custom_switch() {
    let mut extra = Extra::default();
    assert!(!Dependencies::from_settings(&extra).custom_dock_glass_values);
    extra.custom_dock_blur = true;
    assert!(Dependencies::from_settings(&extra).custom_dock_glass_values);
  }

  #[test]
  fn magnification_is_enabled_only_for_magnifying_effects() {
    let mut extra = Extra::default();
    for effect in [
      IconEffect::None,
      IconEffect::Select,
      IconEffect::Light,
      IconEffect::SelectLight,
    ] {
      extra.dock_icon_effect = effect.index() as u8;
      assert!(!Dependencies::from_settings(&extra).dock_magnification);
    }
    for effect in [IconEffect::Scaling, IconEffect::ScalingLight] {
      extra.dock_icon_effect = effect.index() as u8;
      assert!(Dependencies::from_settings(&extra).dock_magnification);
    }
  }
}
