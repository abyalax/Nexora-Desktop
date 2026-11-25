use super::*;

pub(super) fn compact_shelf(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data) {
  let cy = f.h / 2.0;
  p.sym(c, Sym::Tray, f.x0 + p.px(22.0), cy, 15.0, WHITE, 0.9);
  let th = p.px(12.5) * 1.3;
  p.text(
    c,
    &count_text(d.shelf.len()),
    f.x0 + f.w - p.px(16.0),
    cy - th / 2.0,
    12.5,
    600,
    WHITE,
    0.95,
    Anchor::Right,
    f.w / 2.0,
  );
}

pub(super) fn drop_zones(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, regions: &mut Vec<Region>) {
  let (pad, gap) = (p.px(14.0), p.px(10.0));
  let all = [
    (0usize, Sym::Tray, "Simpan ke Rak"),
    (1, Sym::ArrowUp, "Kirim"),
    (2, Sym::Folder, "Salin ke\u{2026}"),
    (3, Sym::Grid, "Buka dengan\u{2026}"),
  ];
  let zones: Vec<_> = all.into_iter().filter(|z| z.0 != 1 || d.send_zone).collect();
  let n = zones.len() as f32;
  let zw = (f.w - 2.0 * pad - (n - 1.0) * gap) / n;
  let (top, zh) = (pad, f.h - 2.0 * pad);
  let accent = crate::prefs::accent();
  for (k, (i, s, label)) in zones.iter().enumerate() {
    let i = *i;
    let x = f.x0 + pad + k as f32 * (zw + gap);
    let on = d.drop_zone == Some(i);
    let fill = if on {
      Color {
        A: 230,
        R: accent[0],
        G: accent[1],
        B: accent[2],
      }
    } else {
      white(22)
    };
    p.rect(c, x, top, zw, zh, p.px(18.0), fill);
    p.sym(
      c,
      *s,
      x + zw / 2.0,
      top + zh / 2.0 - p.px(12.0),
      22.0,
      WHITE,
      if on { 1.0 } else { 0.85 },
    );
    let th = p.px(12.5) * 1.3;
    p.text(
      c,
      tr(label),
      x + zw / 2.0,
      top + zh / 2.0 + p.px(10.0) - th / 2.0 + p.px(6.0),
      12.5,
      600,
      WHITE,
      if on { 1.0 } else { 0.85 },
      Anchor::Center,
      zw - p.px(12.0),
    );
    regions.push(Region {
      x,
      y: top,
      w: zw,
      h: zh,
      action: Action::Zone(i),
      vis: None,
    });
  }
  if d.drop_count > 0 {
    let th = p.px(10.5) * 1.3;
    p.text(
      c,
      &count_text(d.drop_count),
      f.x0 + f.w / 2.0,
      f.h - pad / 2.0 - th / 2.0 - p.px(1.0),
      10.5,
      500,
      WHITE,
      0.55,
      Anchor::Center,
      f.w,
    );
  }
}

pub(super) fn send_card(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, regions: &mut Vec<Region>) {
  let Some(sv) = &d.send else { return };
  let pad = p.px(14.0);
  let (top, h) = (p.px(48.0), f.h - p.px(48.0) - pad);
  let (x0, w) = (f.x0 + pad, f.w - 2.0 * pad);
  card(p, c, x0, top, w, h);
  let s = h - p.px(16.0);
  let (qx, qy) = (x0 + p.px(8.0), top + p.px(8.0));
  if sv.error {
    p.rect(c, qx, qy, s, s, p.px(14.0), white(20));
    p.sym(c, Sym::Wifi, qx + s / 2.0, qy + s / 2.0, 30.0, WHITE, 0.45);
  } else {
    p.rect(
      c,
      qx,
      qy,
      s,
      s,
      p.px(14.0),
      Color {
        A: 255,
        R: 255,
        G: 255,
        B: 255,
      },
    );
    if let Some(q) = sv.qr {
      let inset = ((s - sv.qr_px) / 2.0).round();
      p.place(c, q, qx + inset, qy + inset, sv.qr_px, sv.qr_px, 1.0);
    }
  }
  let tx = qx + s + p.px(18.0);
  let tw = x0 + w - p.px(16.0) - tx;
  let accent = crate::prefs::accent();
  if sv.error {
    p.text(
      c,
      tr("Tidak tersambung ke jaringan lokal"),
      tx,
      top + p.px(16.0),
      14.0,
      700,
      WHITE,
      1.0,
      Anchor::Left,
      tw,
    );
    p.text(
      c,
      tr("Sambungkan laptop ke Wi-Fi yang sama dengan HP."),
      tx,
      top + p.px(40.0),
      12.0,
      500,
      WHITE,
      0.6,
      Anchor::Left,
      tw,
    );
  } else {
    p.text(
      c,
      &p.fit(&sv.title, 14.0, 700, tw),
      tx,
      top + p.px(12.0),
      14.0,
      700,
      WHITE,
      1.0,
      Anchor::Left,
      tw + p.px(4.0),
    );
    p.text(
      c,
      &p.fit(tr("Pindai QR dengan kamera HP (Wi-Fi yang sama)"), 11.5, 500, tw),
      tx,
      top + p.px(33.0),
      11.5,
      500,
      WHITE,
      0.6,
      Anchor::Left,
      tw + p.px(4.0),
    );
    let green = Color {
      A: 255,
      R: GREEN[0],
      G: GREEN[1],
      B: GREEN[2],
    };
    p.text(
      c,
      &sv.status,
      tx,
      top + p.px(55.0),
      12.0,
      600,
      if sv.done { GREEN } else { WHITE },
      0.95,
      Anchor::Left,
      tw,
    );
    let (by, bh) = (top + p.px(76.0), p.px(4.0));
    let bar = tw - p.measure(tr("Selesai"), 12.0, 600) - p.px(28.0 + 16.0);
    p.rect(c, tx, by, bar, bh, bh / 2.0, white(30));
    if sv.frac > 0.0 {
      p.rect(c, tx, by, (bar * sv.frac.clamp(0.0, 1.0)).max(bh), bh, bh / 2.0, green);
    }
    p.text(
      c,
      tr("Izinkan nexora di Windows Firewall bila diminta."),
      tx,
      top + h - p.px(22.0),
      10.0,
      500,
      WHITE,
      0.4,
      Anchor::Left,
      tw - p.px(100.0),
    );
  }
  let label = tr("Selesai");
  let (bw, bh) = (p.measure(label, 12.0, 600) + p.px(28.0), p.px(28.0));
  let (bx, by) = (x0 + w - p.px(14.0) - bw, top + h - p.px(12.0) - bh);
  p.rect(
    c,
    bx,
    by,
    bw,
    bh,
    bh / 2.0,
    Color {
      A: 255,
      R: accent[0],
      G: accent[1],
      B: accent[2],
    },
  );
  let th = p.px(12.0) * 1.3;
  p.text(
    c,
    label,
    bx + bw / 2.0,
    by + bh / 2.0 - th / 2.0,
    12.0,
    600,
    WHITE,
    1.0,
    Anchor::Center,
    bw,
  );
  regions.push(Region {
    x: bx,
    y: by,
    w: bw,
    h: bh,
    action: Action::SendDone,
    vis: None,
  });
}

pub(super) fn shelf_tab(p: &mut Painter, c: &ContainerVisual, f: Frame, d: &Data, regions: &mut Vec<Region>) {
  let pad = p.px(14.0);
  let (top, h) = (p.px(48.0), f.h - p.px(48.0) - pad);
  let (x0, w) = (f.x0 + pad, f.w - 2.0 * pad);
  card(p, c, x0, top, w, h);
  p.text(
    c,
    &count_text(d.shelf.len()),
    x0 + p.px(14.0),
    top + p.px(9.0),
    11.5,
    600,
    WHITE,
    0.5,
    Anchor::Left,
    w / 2.0,
  );
  let clear = tr("Kosongkan");
  let cw = p.measure(clear, 11.5, 600) + p.px(4.0);
  let accent = crate::prefs::accent();
  p.text(
    c,
    clear,
    x0 + w - p.px(14.0),
    top + p.px(9.0),
    11.5,
    600,
    accent,
    0.95,
    Anchor::Right,
    cw + 4.0,
  );
  regions.push(Region {
    x: x0 + w - p.px(14.0) - cw - p.px(6.0),
    y: top + p.px(2.0),
    w: cw + p.px(12.0),
    h: p.px(24.0),
    action: Action::ShelfClear,
    vis: None,
  });
  let (tw, th, gap) = (p.px(96.0), p.px(78.0), p.px(8.0));
  let ty = top + p.px(28.0);
  let max = ((w - p.px(20.0) + gap) / (tw + gap)).floor().max(1.0) as usize;
  let extra = d.shelf.len().saturating_sub(max);
  for (i, (name, icon)) in d.shelf.iter().take(max).enumerate() {
    let x = x0 + p.px(10.0) + i as f32 * (tw + gap);
    if extra > 0 && i == max - 1 {
      p.rect(c, x, ty, tw, th, p.px(14.0), white(14));
      let t = p.px(13.0) * 1.3;
      p.text(
        c,
        &format!("+{}", extra + 1),
        x + tw / 2.0,
        ty + th / 2.0 - t / 2.0,
        13.0,
        600,
        WHITE,
        0.7,
        Anchor::Center,
        tw,
      );
      break;
    }
    let n0 = kids(c);
    let isz = p.px(40.0);
    match icon {
      Some(s) => {
        p.image(c, s, x + (tw - isz) / 2.0, ty + p.px(6.0), isz, isz, p.px(6.0));
      }
      None => p.sym(c, Sym::Folder, x + tw / 2.0, ty + p.px(26.0), 26.0, WHITE, 0.7),
    }
    let shown = p.fit(name, 11.0, 500, tw - p.px(8.0));
    p.text(
      c,
      &shown,
      x + tw / 2.0,
      ty + p.px(52.0),
      11.0,
      500,
      WHITE,
      0.9,
      Anchor::Center,
      tw - p.px(6.0),
    );
    regions.push(Region {
      x,
      y: ty,
      w: tw,
      h: th,
      action: Action::ShelfItem(i),
      vis: group_since(c, n0),
    });
  }
}

pub(super) fn shelf_empty(p: &mut Painter, c: &ContainerVisual, f: Frame) {
  let pad = p.px(14.0);
  let (top, h) = (p.px(48.0), f.h - p.px(48.0) - pad);
  card(p, c, f.x0 + pad, top, f.w - 2.0 * pad, h);
  let cx = f.x0 + f.w / 2.0;
  p.sym(c, Sym::Plus, cx, top + h / 2.0 - p.px(16.0), 18.0, WHITE, 0.4);
  p.text(
    c,
    tr("Rak kosong"),
    cx,
    top + h / 2.0 + p.px(2.0),
    13.0,
    600,
    WHITE,
    0.8,
    Anchor::Center,
    f.w,
  );
  p.text(
    c,
    tr("Seret berkas ke Visor untuk menaruhnya di sini."),
    cx,
    top + h / 2.0 + p.px(22.0),
    12.0,
    400,
    WHITE,
    0.5,
    Anchor::Center,
    f.w,
  );
}
