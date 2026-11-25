use super::*;

pub(super) fn compact_timer(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, urgent: bool) {
  let cy = f.h / 2.0;
  let rgb = if urgent { ORANGE } else { WHITE };
  p.sym(c, Sym::Stopwatch, f.x0 + p.px(20.0), cy, 15.0, rgb, 0.95);
  let t = format_mmss(d.timer.remaining_secs(d.now_ms));
  let th = p.px(13.0) * 1.3;
  p.text(
    c,
    &t,
    f.x0 + f.w - p.px(16.0),
    cy - th / 2.0,
    13.0,
    600,
    rgb,
    1.0,
    Anchor::Right,
    f.w,
  );
}

pub(super) fn round_button(p: &mut Painter, c: &ContainerVisual, cx: f32, cy: f32, d: f32, fill: Color) {
  p.rect(c, cx - d / 2.0, cy - d / 2.0, d, d, d / 2.0, fill);
}

pub(super) fn timer(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, regions: &mut Vec<Region>) {
  let (pad, gap) = (p.px(14.0), p.px(10.0));
  let (top, h) = (p.px(48.0), f.h - p.px(48.0) - pad);
  let cw = (f.w - 2.0 * pad - gap) / 2.0;
  timer_card(p, c, f.x0 + pad, top, cw, h, d, regions);
  calendar_card(p, c, f.x0 + pad + cw + gap, top, cw, h, d, regions);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn timer_card(
  p: &mut Painter,
  c: &ContainerVisual,
  x: f32,
  y: f32,
  w: f32,
  h: f32,
  d: &Data,
  regions: &mut Vec<Region>,
) {
  card(p, c, x, y, w, h);
  let t = d.timer;
  let secs = t.remaining_secs(d.now_ms);
  let (rgb, status) = match t.phase {
    Phase::Running if secs <= 60 => (ORANGE, tr("Berjalan")),
    Phase::Running => (WHITE, tr("Berjalan")),
    Phase::Paused => (WHITE, tr("Dijeda")),
    Phase::Finished => (ORANGE, tr("Selesai")),
    Phase::Idle => (WHITE, tr("Siap")),
  };
  p.text(
    c,
    &format_mmss(secs),
    x + p.px(18.0),
    y + p.px(8.0),
    30.0,
    600,
    rgb,
    1.0,
    Anchor::Left,
    w * 0.7,
  );
  p.text(
    c,
    status,
    x + w - p.px(16.0),
    y + p.px(18.0),
    11.5,
    500,
    WHITE,
    0.55,
    Anchor::Right,
    w * 0.3,
  );
  let (bd, bgap) = (p.px(34.0), p.px(14.0));
  let total = 4.0 * bd + 3.0 * bgap;
  let cy = y + h - p.px(12.0) - bd / 2.0;
  let mut cx = x + (w - total) / 2.0 + bd / 2.0;
  let idle = matches!(t.phase, Phase::Idle | Phase::Finished);
  let accent = crate::prefs::accent();
  let buttons: [(Action, bool); 4] = [
    (Action::TimerMinus, idle),
    (Action::TimerToggle, true),
    (Action::TimerReset, true),
    (Action::TimerPlus, idle),
  ];
  for (a, enabled) in buttons {
    let primary = a == Action::TimerToggle;
    let fill = if primary {
      Color {
        A: 255,
        R: accent[0],
        G: accent[1],
        B: accent[2],
      }
    } else {
      white(if enabled { 30 } else { 14 })
    };
    let n0 = kids(c);
    round_button(p, c, cx, cy, bd, fill);
    let op = if enabled { 1.0 } else { 0.35 };
    match a {
      Action::TimerToggle => p.sym(
        c,
        if t.phase == Phase::Running {
          Sym::Pause
        } else {
          Sym::Play
        },
        cx,
        cy,
        16.0,
        WHITE,
        1.0,
      ),
      Action::TimerReset => p.sym(c, Sym::Restart, cx, cy, 15.0, WHITE, 0.9),
      Action::TimerMinus | Action::TimerPlus => {
        let th = p.px(12.0) * 1.3;
        let s = if a == Action::TimerMinus { "\u{2212}5" } else { "+5" };
        p.text(c, s, cx, cy - th / 2.0, 12.0, 600, WHITE, op, Anchor::Center, bd);
      }
      _ => {}
    }
    if enabled {
      regions.push(Region {
        x: cx - bd / 2.0,
        y: cy - bd / 2.0,
        w: bd,
        h: bd,
        action: a,
        vis: group_since(c, n0),
      });
    }
    cx += bd + bgap;
  }
}
