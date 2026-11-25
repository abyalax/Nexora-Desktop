//! Resolve [`Settings`] and a [`Material`] into the concrete glass [`Look`] used by every surface.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
  pub dark: bool,
  pub blur_sigma: f32,
  pub saturation: f32,
  pub tint: [u8; 4],
  pub rim: [u8; 4],
  pub solid: [u8; 4],
  pub highlight: f32,
  pub fg: [u8; 3],
  pub liquify: bool,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub fn look(s: &Settings, system_dark: bool, material: Material) -> Look {
  let dark = match (material, s.theme) {
    (Material::Black, _) => true,
    (_, Theme::Auto) => system_dark,
    (_, Theme::Dark) => true,
    (_, Theme::Light) => false,
  };
  let b = s.blur.min(100) as f32 / 100.0;
  let t = s.transparency.min(100) as f32 / 100.0;
  let (sigma_k, sat) = match s.style {
    BlurStyle::Frosted => (1.0, 1.25),
    BlurStyle::Clear => (0.45, 1.05),
    BlurStyle::Vivid => (0.85, 1.9),
  };
  let mut blur_sigma = lerp(1.0, 40.0, b) * sigma_k;
  let mut saturation: f32 = sat;
  let (tr, tg, tb) = if dark { (20u8, 20u8, 24u8) } else { (255, 255, 255) };
  let a: f32;
  let highlight: f32;
  let fg = if dark { [255u8, 255, 255] } else { [24, 24, 28] };
  let (mut tr, mut tg, mut tb) = (tr, tg, tb);
  let rim: [u8; 4];
  let solid: [u8; 4];
  match material {
    Material::Glass => {
      a = if dark {
        lerp(120.0, 22.0, t)
      } else {
        lerp(80.0, 18.0, t)
      };
      if !dark {
        (tr, tg, tb) = (226, 231, 240);
      }
      blur_sigma = lerp(1.5, 7.0, b) * sigma_k;
      saturation = saturation.max(1.4);
      rim = [140, 255, 255, 255];
      solid = if dark {
        [235u8, 28, 28, 32]
      } else {
        [235, 238, 240, 246]
      };
      highlight = 0.35;
    }
    Material::Blur => {
      a = if dark {
        lerp(100.0, 30.0, t)
      } else {
        lerp(120.0, 40.0, t)
      };
      (tr, tg, tb) = if dark { (26, 28, 30) } else { (240, 242, 246) };
      blur_sigma = lerp(8.0, 36.0, b);
      saturation = 1.1;
      rim = if dark { [34, 255, 255, 255] } else { [60, 255, 255, 255] };
      solid = if dark {
        [235u8, 28, 28, 32]
      } else {
        [235, 238, 240, 246]
      };
      highlight = 0.2;
    }
    Material::Black => {
      (tr, tg, tb) = (10, 10, 12);
      a = lerp(255.0, 225.0, t);
      saturation = 1.0;
      blur_sigma *= 0.8;
      rim = [0, 255, 255, 255];
      solid = [255, 10, 10, 12];
      highlight = 0.0;
    }
  }
  let liquify = material == Material::Glass && s.refraction;
  Look {
    dark,
    blur_sigma,
    saturation,
    tint: [a.round() as u8, tr, tg, tb],
    rim,
    solid,
    highlight,
    fg,
    liquify,
  }
}

pub fn bar_look(s: &Settings, system_dark: bool) -> Look {
  let mut t = *s;
  match s.menubar_style {
    BarStyle::Light => t.theme = Theme::Light,
    BarStyle::Dark => t.theme = Theme::Dark,
  }
  let mut l = look(&t, system_dark, Material::Glass);
  l.tint[0] = 238;
  l.blur_sigma = l.blur_sigma.min(14.0);
  l.saturation = 1.0;
  l.highlight = 0.25;
  l.rim[0] = l.rim[0].min(60);
  l.liquify = false;
  l
}
