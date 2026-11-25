use super::*;

pub(super) fn compact_glance(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data) {
  let Some((item, text)) = &d.glance else {
    return;
  };
  let cy = f.h / 2.0;
  let th = p.px(12.5) * 1.3;
  if *item == Item::Greeting {
    let max = f.w - p.px(32.0);
    let shown = p.fit(text, 12.5, 600, max);
    p.text(
      c,
      &shown,
      f.x0 + f.w / 2.0,
      cy - th / 2.0,
      12.5,
      600,
      WHITE,
      0.95,
      Anchor::Center,
      max + p.px(2.0),
    );
    return;
  }
  let left = f.x0 + p.px(14.0);
  let icon_w = match item {
    Item::Battery => {
      let (pct, ch) = d.battery.unwrap_or((0, false));
      p.battery(c, left, cy, pct, ch)
    }
    _ => {
      let sym = if *item == Item::Clock {
        Sym::Clock
      } else {
        Sym::Calendar
      };
      p.sym(c, sym, left + p.px(8.0), cy, 15.0, WHITE, 0.9);
      p.px(16.0)
    }
  };
  let right = f.x0 + f.w - p.px(16.0);
  let max = right - (left + icon_w + p.px(10.0));
  let shown = p.fit(text, 12.5, 600, max);
  p.text(
    c,
    &shown,
    right,
    cy - th / 2.0,
    12.5,
    600,
    WHITE,
    0.95,
    Anchor::Right,
    max + p.px(2.0),
  );
}

pub(super) fn compact_download(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data) {
  let Some(dl) = &d.download else { return };
  let cy = f.h / 2.0;
  let a = p.px(22.0);
  let ix = f.x0 + p.px(14.0);
  match dl.icon {
    Some(s) => {
      p.image(c, s, ix, cy - a / 2.0, a, a, p.px(6.0));
    }
    None => {
      p.rect(c, ix, cy - a / 2.0, a, a, a / 2.0, white(36));
      p.sym(
        c,
        if dl.up { Sym::ArrowUp } else { Sym::ArrowDown },
        ix + a / 2.0,
        cy,
        11.0,
        WHITE,
        0.9,
      );
    }
  }
  let (r, w) = (11.0, 2.6);
  let gx = f.x0 + f.w - p.px(16.0) - p.px(r + w / 2.0);
  let gy = p.px(15.0);
  p.arc(c, gx, gy, r, w, 0.0, 1.0, WHITE, 56);
  match (dl.frac, dl.done) {
    (_, true) => {
      p.arc(c, gx, gy, r, w, 0.0, 1.0, GREEN, 255);
    }
    (Some(fr), false) => {
      if fr > 0.002 {
        p.arc(c, gx, gy, r, w, 0.0, fr.clamp(0.0, 1.0), GREEN, 255);
      }
    }
    (None, false) => {
      if let Some((v, n)) = p.arc(c, gx, gy, r, w, 0.0, 0.24, GREEN, 255) {
        spin(p, &v, n, d.now_ms);
      }
    }
  }
  p.sym(
    c,
    if dl.done {
      Sym::Check
    } else if dl.up {
      Sym::ArrowUp
    } else {
      Sym::ArrowDown
    },
    gx,
    gy,
    12.0,
    WHITE,
    0.95,
  );
  let th = p.px(9.5) * 1.3;
  p.text(
    c,
    &dl.label,
    gx,
    f.h - p.px(2.0) - th,
    9.5,
    600,
    WHITE,
    0.95,
    Anchor::Center,
    p.px(40.0),
  );
}

pub(super) fn spin(p: &Painter, v: &SpriteVisual, n: f32, now_ms: i64) {
  use windows::Foundation::TimeSpan;
  use windows::UI::Composition::{AnimationIterationBehavior, CompositionEasingFunction};
  use windows_core::{HSTRING, Interface};
  const PERIOD: i64 = 1100;
  let go = || -> windows_core::Result<()> {
    let phase = (now_ms.rem_euclid(PERIOD)) as f32 / PERIOD as f32 * 360.0;
    v.SetCenterPoint(Vector3 {
      X: n / 2.0,
      Y: n / 2.0,
      Z: 0.0,
    })?;
    let lin: CompositionEasingFunction = p.comp.CreateLinearEasingFunction()?.cast()?;
    let a = p.comp.CreateScalarKeyFrameAnimation()?;
    a.InsertKeyFrame(0.0, phase)?;
    a.InsertKeyFrameWithEasingFunction(1.0, phase + 360.0, &lin)?;
    a.SetDuration(TimeSpan {
      Duration: PERIOD * 10_000,
    })?;
    a.SetIterationBehavior(AnimationIterationBehavior::Forever)?;
    v.StartAnimation(&HSTRING::from("RotationAngleInDegrees"), &a)
  };
  let _ = go();
}

pub(super) fn compact_charging(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data) {
  let cy = f.h / 2.0;
  let th = p.px(13.0) * 1.3;
  p.text(
    c,
    tr("Mengisi daya"),
    f.x0 + p.px(16.0),
    cy - th / 2.0,
    12.5,
    500,
    WHITE,
    0.85,
    Anchor::Left,
    f.w / 2.0,
  );
  let pct = d.battery.map_or(String::new(), |(b, _)| format!("\u{26A1} {b}%"));
  p.text(
    c,
    &pct,
    f.x0 + f.w - p.px(16.0),
    cy - th / 2.0,
    13.0,
    600,
    GREEN,
    1.0,
    Anchor::Right,
    f.w / 2.0,
  );
}

pub(super) fn header(p: &mut Painter, c: &ContainerVisual, f: Frame, cur: Tab, d: &Data, regions: &mut Vec<Region>) {
  let (cy, bh, pad) = (p.px(25.0), p.px(26.0), p.px(3.0));
  let tabs = nexora_core::dynamic_island::activities::Activities::from_settings(&crate::prefs::get()).visible_tabs();
  let widths: Vec<f32> = tabs
    .iter()
    .map(|t| (p.measure(tr(t.title()), 12.5, 600) + p.px(28.0)).max(p.px(60.0)))
    .collect();
  let total: f32 = widths.iter().sum::<f32>() + 2.0 * pad;
  let x = f.x0 + p.px(16.0);
  p.rect(c, x, cy - bh / 2.0, total, bh, bh / 2.0, white(26));
  let mut sx = x + pad;
  let th = p.px(12.5) * 1.3;
  for (t, w) in tabs.iter().zip(&widths) {
    let on = *t == cur;
    let n0 = kids(c);
    if on {
      p.rect(
        c,
        sx,
        cy - bh / 2.0 + pad,
        *w,
        bh - 2.0 * pad,
        (bh - 2.0 * pad) / 2.0,
        white(236),
      );
    }
    p.text(
      c,
      tr(t.title()),
      sx + w / 2.0,
      cy - th / 2.0,
      12.5,
      600,
      if on { INK } else { WHITE },
      if on { 1.0 } else { 0.78 },
      Anchor::Center,
      *w,
    );
    regions.push(Region {
      x: sx,
      y: cy - bh / 2.0 - pad,
      w: *w,
      h: bh + 2.0 * pad,
      action: Action::Tab(*t),
      vis: group_since(c, n0),
    });
    sx += w;
  }
  let gx = f.x0 + f.w - p.px(28.0);
  let n0 = kids(c);
  p.sym(c, Sym::Gear, gx, cy, 16.0, WHITE, 0.7);
  regions.push(Region {
    x: gx - p.px(16.0),
    y: cy - p.px(15.0),
    w: p.px(32.0),
    h: p.px(30.0),
    action: Action::Settings,
    vis: group_since(c, n0),
  });
  if let Some((pct, charging)) = d.battery {
    let label = if charging {
      format!("\u{26A1}{pct}%")
    } else {
      format!("{pct}%")
    };
    p.text(
      c,
      &label,
      gx - p.px(18.0),
      cy - th / 2.0,
      12.5,
      600,
      if charging { GREEN } else { WHITE },
      0.82,
      Anchor::Right,
      p.px(80.0),
    );
  }
}
