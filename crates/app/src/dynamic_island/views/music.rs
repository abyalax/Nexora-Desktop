use super::*;

pub(super) fn accent_or(t: Option<[u8; 3]>) -> [u8; 3] {
  t.unwrap_or_else(crate::prefs::accent)
}

pub(super) fn current_line(d: &Data) -> String {
  match d.lyrics.lines {
    Some(l) => l.current(d.lyrics.pos_ms).unwrap_or("\u{266A}").to_string(),
    None => String::new(),
  }
}

pub(super) fn compact_lyrics(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, live: &mut Live) {
  compact_music(p, c, f, d, live, f.h);
  let left = f.x0 + p.px(12.0 + 20.0 + 10.0);
  let right = f.x0 + f.w - p.px(14.0 + 27.0 + 10.0);
  let max = (right - left).max(p.px(20.0));
  let line = current_line(d);
  let shown = p.fit(&line, 13.0, 600, max);
  let th = p.px(13.0) * 1.3;
  let (_, v) = p.text_v(
    c,
    &shown,
    (left + right) / 2.0,
    f.h / 2.0 - th / 2.0,
    13.0,
    600,
    WHITE,
    1.0,
    Anchor::Center,
    max + p.px(2.0),
  );
  live.lyric = v;
}

pub(super) fn compact_player(
  p: &mut Painter,
  c: &ContainerVisual,
  f: Frame,
  d: &Data,
  regions: &mut Vec<Region>,
  live: &mut Live,
) {
  let Some(m) = &d.media else { return };
  let a = p.px(52.0);
  let (ax, ay) = (f.x0 + p.px(22.0), p.px(10.0));
  match m.art {
    Some(s) => {
      p.image(c, s, ax, ay, a, a, p.px(12.0));
    }
    None => {
      p.rect(c, ax, ay, a, a, p.px(12.0), white(36));
      p.sym(c, Sym::Note, ax + a / 2.0, ay + a / 2.0, 20.0, WHITE, 0.7);
    }
  }
  let (cx0, cy, step, hit) = (ax + a / 2.0, ay + a + p.px(17.0), p.px(24.0), p.px(24.0));
  let buttons = [
    (Action::Prev, Sym::Backward, 12.0),
    (Action::PlayPause, if m.playing { Sym::Pause } else { Sym::Play }, 14.0),
    (Action::Next, Sym::Forward, 12.0),
  ];
  for (i, (act, sym, size)) in buttons.into_iter().enumerate() {
    let cx = cx0 + (i as f32 - 1.0) * step;
    let n0 = kids(c);
    p.sym(c, sym, cx, cy, size, WHITE, 0.95);
    regions.push(Region {
      x: cx - hit / 2.0,
      y: cy - hit / 2.0,
      w: hit,
      h: hit,
      action: act,
      vis: group_since(c, n0),
    });
  }
  let lx = ax + a + p.px(28.0);
  let lw = f.x0 + f.w - p.px(22.0) - lx;
  let synced = d.lyrics.lines.is_some_and(|l| l.synced);
  if synced {
    lyrics_flow(
      p,
      c,
      lx,
      p.px(6.0),
      lw,
      f.h - p.px(12.0),
      d,
      live,
      Flow {
        size: 16.0,
        pad: 0.0,
        anchor: 0.36,
        card: false,
      },
    );
  } else {
    let th = p.px(16.0) * 1.3;
    let cy = f.h / 2.0;
    p.text(
      c,
      &p.fit(m.title, 16.0, 700, lw),
      lx,
      cy - th,
      16.0,
      700,
      WHITE,
      1.0,
      Anchor::Left,
      lw + p.px(4.0),
    );
    p.text(
      c,
      &p.fit(m.artist, 13.0, 500, lw),
      lx,
      cy + p.px(2.0),
      13.0,
      500,
      WHITE,
      0.6,
      Anchor::Left,
      lw + p.px(4.0),
    );
  }
}

pub(super) fn compact_music(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, live: &mut Live, row_h: f32) {
  let Some(m) = &d.media else { return };
  let cy = row_h / 2.0;
  let a = p.px(20.0);
  match m.art {
    Some(s) => {
      p.image(c, s, f.x0 + p.px(12.0), cy - a / 2.0, a, a, p.px(6.0));
    }
    None => {
      p.rect(c, f.x0 + p.px(12.0), cy - a / 2.0, a, a, p.px(6.0), white(40));
      p.sym(c, Sym::Note, f.x0 + p.px(12.0) + a / 2.0, cy, 12.0, WHITE, 0.8);
    }
  }
  let rgb = accent_or(m.tint);
  let (bw, gap, bh) = (p.px(3.0), p.px(3.0), p.px(14.0));
  let right = f.x0 + f.w - p.px(14.0);
  let x0 = right - 5.0 * bw - 4.0 * gap;
  for i in 0..5 {
    if let Some(v) = p.rect(
      c,
      x0 + i as f32 * (bw + gap),
      cy - bh / 2.0,
      bw,
      bh,
      bw / 2.0,
      Color {
        A: 240,
        R: rgb[0],
        G: rgb[1],
        B: rgb[2],
      },
    ) {
      let _ = v.SetCenterPoint(Vector3 {
        X: bw / 2.0,
        Y: bh / 2.0,
        Z: 0.0,
      });
      let _ = v.SetScale(Vector3 { X: 1.0, Y: 0.3, Z: 1.0 });
      live.bars.push(v);
    }
  }
}

pub(super) fn mmss(ms: i64) -> String {
  let s = (ms.max(0) + 500) / 1000;
  format!("{}:{:02}", s / 60, s % 60)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn music_card(
  p: &mut Painter,
  c: &ContainerVisual,
  x: f32,
  y: f32,
  w: f32,
  h: f32,
  m: &MediaView,
  regions: &mut Vec<Region>,
) {
  let tint = m.tint;
  let bg = match tint {
    Some(t) => Color {
      A: 48,
      R: t[0],
      G: t[1],
      B: t[2],
    },
    None => white(18),
  };
  p.rect(c, x, y, w, h, p.px(18.0), bg);
  let pad = p.px(14.0);
  let art = p.px(64.0);
  let (ax, ay) = (x + pad, y + pad);
  let glow = accent_or(tint);
  p.rect(
    c,
    ax - p.px(4.0),
    ay - p.px(2.0),
    art + p.px(8.0),
    art + p.px(8.0),
    p.px(18.0),
    Color {
      A: 60,
      R: glow[0],
      G: glow[1],
      B: glow[2],
    },
  );
  match m.art {
    Some(s) => {
      p.image(c, s, ax, ay, art, art, p.px(14.0));
    }
    None => {
      p.rect(c, ax, ay, art, art, p.px(14.0), white(40));
      p.sym(c, Sym::Note, ax + art / 2.0, ay + art / 2.0, 24.0, WHITE, 0.8);
    }
  }
  let tx = ax + art + p.px(12.0);
  let tw = x + w - pad - tx;
  let title = p.fit(m.title, 13.5, 600, tw);
  p.text(
    c,
    &title,
    tx,
    ay + p.px(10.0),
    13.5,
    600,
    WHITE,
    1.0,
    Anchor::Left,
    tw + 2.0,
  );
  let artist = p.fit(m.artist, 12.0, 400, tw);
  p.text(
    c,
    &artist,
    tx,
    ay + p.px(30.0),
    12.0,
    400,
    WHITE,
    0.62,
    Anchor::Left,
    tw + 2.0,
  );

  let (sx, sw) = (x + pad, w - 2.0 * pad);
  let sy = ay + art + p.px(14.0);
  let th = p.px(5.0);
  let frac = m
    .drag
    .unwrap_or(if m.dur_ms > 0 {
      m.pos_ms as f32 / m.dur_ms as f32
    } else {
      0.0
    })
    .clamp(0.0, 1.0);
  p.rect(c, sx, sy - th / 2.0, sw, th, th / 2.0, white(52));
  p.rect(c, sx, sy - th / 2.0, (sw * frac).max(th), th, th / 2.0, white(236));
  let kx = sx + sw * frac;
  if m.drag.is_some() {
    let (kw, kh) = (p.px(28.0), p.px(18.0));
    p.rect(c, kx - kw / 2.0, sy - kh / 2.0, kw, kh, kh / 2.0, white(125));
    p.rect(
      c,
      kx - kw / 2.0 + p.px(1.5),
      sy - kh / 2.0 + p.px(1.5),
      kw - p.px(3.0),
      kh - p.px(3.0),
      (kh - p.px(3.0)) / 2.0,
      white(40),
    );
  } else {
    let (kw, kh) = (p.px(18.0), p.px(12.0));
    p.rect(c, kx - kw / 2.0, sy - kh / 2.0, kw, kh, kh / 2.0, white(250));
  }
  if m.can_seek && m.dur_ms > 0 {
    regions.push(Region {
      x: sx - p.px(6.0),
      y: sy - p.px(12.0),
      w: sw + p.px(12.0),
      h: p.px(24.0),
      action: Action::Seek(sx, sw),
      vis: None,
    });
  }
  let pos = if let Some(fr) = m.drag {
    (m.dur_ms as f32 * fr) as i64
  } else {
    m.pos_ms
  };
  let ty = sy + p.px(8.0);
  p.text(c, &mmss(pos), sx, ty, 10.5, 500, WHITE, 0.55, Anchor::Left, sw / 2.0);
  p.text(
    c,
    &format!("-{}", mmss(m.dur_ms - pos)),
    sx + sw,
    ty,
    10.5,
    500,
    WHITE,
    0.55,
    Anchor::Right,
    sw / 2.0,
  );

  let cy = y + h - p.px(20.0);
  let cx = x + w / 2.0;
  let gap = p.px(48.0);
  let items = [
    (Sym::Backward, cx - gap, 17.0, Action::Prev),
    (
      if m.playing { Sym::Pause } else { Sym::Play },
      cx,
      22.0,
      Action::PlayPause,
    ),
    (Sym::Forward, cx + gap, 17.0, Action::Next),
  ];
  for (s, bx, size, a) in items {
    let n0 = kids(c);
    p.sym(c, s, bx, cy, size, WHITE, 0.95);
    regions.push(Region {
      x: bx - p.px(20.0),
      y: cy - p.px(16.0),
      w: p.px(40.0),
      h: p.px(32.0),
      action: a,
      vis: group_since(c, n0),
    });
  }
}
