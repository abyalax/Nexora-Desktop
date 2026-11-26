use super::*;

pub(super) fn play(
  p: &mut Painter,
  c: &ContainerVisual,
  f: Frame,
  d: &Data,
  regions: &mut Vec<Region>,
  live: &mut Live,
) {
  let (pad, gap) = (p.px(14.0), p.px(10.0));
  let (top, bottom) = (p.px(48.0), f.h - pad);
  let h = bottom - top;
  let mw = p.px(230.0);
  let lw = f.w - 2.0 * pad - mw - gap;
  let (mx, lx) = (f.x0 + pad, f.x0 + pad + mw + gap);
  match &d.media {
    Some(m) => music_card(p, c, mx, top, mw, h, m, regions),
    None => {
      card(p, c, mx, top, mw, h);
      p.sym(
        c,
        Sym::Note,
        mx + mw / 2.0,
        top + h / 2.0 - p.px(14.0),
        26.0,
        WHITE,
        0.35,
      );
      p.text(
        c,
        tr("Tidak diputar"),
        mx + mw / 2.0,
        top + h / 2.0 + p.px(8.0),
        12.5,
        500,
        WHITE,
        0.6,
        Anchor::Center,
        mw - p.px(20.0),
      );
    }
  }
  lyrics_flow(
    p,
    c,
    lx,
    top,
    lw,
    h,
    d,
    live,
    Flow {
      size: 17.0,
      pad: 16.0,
      anchor: 0.30,
      card: true,
    },
  );
}

#[derive(Clone, Copy)]
pub(super) struct Flow {
  pub size: f32,
  pub pad: f32,
  pub anchor: f32,
  pub card: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn lyrics_flow(
  p: &mut Painter,
  c: &ContainerVisual,
  x: f32,
  y: f32,
  w: f32,
  h: f32,
  d: &Data,
  live: &mut Live,
  fl: Flow,
) {
  if fl.card {
    card(p, c, x, y, w, h);
  }
  let ly = &d.lyrics;
  let note = match ly.lines {
    Some(l) if !l.synced => tr("Lirik tanpa waktu"),
    Some(_) if ly.lrclib => "LRCLIB",
    _ => "",
  };
  let center_msg = |p: &mut Painter, s: &str| {
    p.text(
      c,
      s,
      x + w / 2.0,
      y + h / 2.0 - p.px(8.0),
      13.0,
      600,
      WHITE,
      0.45,
      Anchor::Center,
      w - p.px(20.0),
    );
  };
  let Some(l) = ly.lines else {
    match (ly.state, d.media.is_some()) {
      (LyrState::Loading, _) => center_msg(p, tr("Mencari lirik\u{2026}")),
      (_, true) => center_msg(p, tr("Lirik tidak tersedia")),
      _ => center_msg(p, tr("Lirik tampil di sini saat lagu diputar.")),
    }
    return;
  };
  let go = || -> windows_core::Result<(ContainerVisual, ContainerVisual)> {
    let frame = p.comp.CreateContainerVisual()?;
    frame.SetOffset(Vector3 { X: x, Y: y, Z: 0.0 })?;
    frame.SetSize(windows_numerics::Vector2 { X: w, Y: h })?;
    frame.SetClip(&p.comp.CreateInsetClip()?)?;
    c.Children()?.InsertAtTop(&frame)?;
    let scroll = p.comp.CreateContainerVisual()?;
    scroll.SetSize(windows_numerics::Vector2 { X: w, Y: h })?;
    frame.Children()?.InsertAtTop(&scroll)?;
    Ok((frame, scroll))
  };
  let Ok((frame, sc)) = go() else { return };
  let (pad, size) = (p.px(fl.pad), fl.size);
  let line_h = p.px(size) * 1.28;
  let gap = p.px(size * 0.53);
  let maxw = w - 2.0 * pad;
  let blocks = |p: &Painter, text: &str| -> Vec<String> {
    let t = if text.is_empty() { "\u{266A}" } else { text };
    nexora_core::dynamic_island::typeset::wrap(t, maxw, 3, |s| p.measure(s, size, 700))
  };
  if l.synced {
    let active = l.index_at(ly.pos_ms);
    let center = active.unwrap_or(0);
    let ya = h * fl.anchor;
    let mut drawn: Vec<(usize, f32, Vec<String>)> = Vec::new();
    if center > 0 {
      let prev = blocks(p, &l.lines[center - 1].text);
      let hp = prev.len() as f32 * line_h;
      live.lyric_shift = hp + gap;
      drawn.push((center - 1, ya - gap - hp, prev));
    }
    let mut yy = ya;
    for i in center..l.lines.len() {
      if yy > h {
        break;
      }
      let b = blocks(p, &l.lines[i].text);
      let hb = b.len() as f32 * line_h;
      drawn.push((i, yy, b));
      yy += hb + gap;
    }
    for (i, top, lines) in drawn {
      let on = active == Some(i);
      for (k, t) in lines.iter().enumerate() {
        let ty = top + k as f32 * line_h;
        if on {
          p.text(&sc, t, pad, ty, size, 700, WHITE, 1.0, Anchor::Left, maxw + p.px(4.0));
        } else {
          p.text_soft(&sc, t, pad, ty, size, 700, WHITE, 0.32, (2.2 * p.dpi).round());
        }
      }
    }
    live.lyric_scroll = Some(sc);
  } else {
    let mut yy = pad;
    for line in l.lines.iter().filter(|x| !x.text.is_empty()) {
      if yy > h {
        break;
      }
      for t in nexora_core::dynamic_island::typeset::wrap(&line.text, maxw, 3, |s| p.measure(s, 15.0, 600)) {
        p.text(&sc, &t, pad, yy, 15.0, 600, WHITE, 0.8, Anchor::Left, maxw + p.px(4.0));
        yy += p.px(15.0) * 1.28;
      }
      yy += gap * 0.6;
    }
  }
  if fl.card {
    p.text(
      &frame,
      note,
      w - p.px(12.0),
      h - p.px(18.0),
      9.5,
      500,
      WHITE,
      0.35,
      Anchor::Right,
      w / 2.0,
    );
  }
}
