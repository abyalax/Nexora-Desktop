use super::*;

impl Mgr {
  pub(super) fn open_gallery(&mut self) {
    if self.gallery.is_some() {
      return;
    }
    let d = self.dpi;
    let (gw, gh) = (760.0 * d, 500.0 * d);
    let mut glass = prefs::glass((gw, gh), 26.0 * d, prefs::Role::Themed);
    glass.blur_sigma = glass.blur_sigma.max(22.0);
    let Ok(mut popup) = Popup::new_with(
      &self.comp,
      w!("NexoraWidgetGallery"),
      Some(gallery_proc),
      WINDOW_EX_STYLE(0),
      glass,
      true,
    ) else {
      return;
    };
    let (sw, sh) = sys::screen_size();
    let _ = popup.set_rect(
      ((sw as f32 - gw) / 2.0) as i32,
      (sh as f32 - gh - 110.0 * d).max(40.0 * d) as i32,
      gw as i32,
      gh as i32,
    );
    let _ = popup.panel.set_radius(26.0 * d);
    self.gallery = Some(Gallery {
      popup,
      cat: None,
      query: String::new(),
      cards: vec![],
      cats: vec![],
      done: (0.0, 0.0, 0.0, 0.0),
      conts: vec![],
      hover: None,
      tracking: false,
      press: None,
      closing: false,
    });
    self.refresh_gallery();
    self.set_edit(true);
    if let Some(g) = &self.gallery {
      g.popup.show_active();
      let _ = motion::pop_in(
        &self.comp,
        g.popup.root(),
        Vector3 {
          X: gw / 2.0,
          Y: gh,
          Z: 0.0,
        },
        0.82,
        1.02,
        0,
        340,
      );
    }
  }

  pub(super) fn close_gallery(&mut self) {
    let Some(g) = self.gallery.as_mut() else {
      return;
    };
    if g.closing {
      return;
    }
    g.closing = true;
    let (gw, gh) = (g.popup.rect.2 as f32, g.popup.rect.3 as f32);
    let _ = motion::pop_out(
      &self.comp,
      g.popup.root(),
      Vector3 {
        X: gw / 2.0,
        Y: gh,
        Z: 0.0,
      },
      0.86,
      180,
    );
    unsafe {
      SetTimer(Some(self.ctl), T_GCLOSE, 190, None);
    }
    self.set_edit(false);
  }

  pub(super) fn finish_close_gallery(&mut self) {
    unsafe {
      let _ = KillTimer(Some(self.ctl), T_GCLOSE);
    }
    if let Some(g) = self.gallery.take() {
      unsafe {
        let _ = DestroyWindow(g.popup.hwnd);
      }
      self.cache.clear();
      nexora_platform::sys::trim_later(5000);
    }
  }

  pub(super) fn refresh_gallery(&mut self) {
    let Some(g) = self.gallery.as_ref() else {
      return;
    };
    let d = self.dpi;
    let (cat, raw_query) = (g.cat, g.query.clone());
    let query = raw_query.to_lowercase();
    let content = g.popup.content.clone();
    let (gw, gh) = (g.popup.rect.2 as f32, g.popup.rect.3 as f32);
    if let Ok(k) = content.Children() {
      let _ = k.RemoveAll();
    }
    let look = prefs::look(prefs::Role::Themed);
    let fg = look.fg;
    let side = 200.0 * d;
    let mut conts: Vec<(Rect, ContainerVisual, f32)> = Vec::new();
    let _ = self.rr(&content, (side, 0.0, 1.0, gh), 0.0, col(fg, 26));
    let sb: Rect = (14.0 * d, 16.0 * d, side - 28.0 * d, 30.0 * d);
    let _ = self.rr(&content, sb, 9.0 * d, col(fg, 22));
    self.glyph(
      &content,
      "\u{E721}",
      11.0 * d,
      fg,
      sb.0 + 14.0 * d,
      sb.1 + sb.3 / 2.0,
      0.6,
    );
    let q = if query.is_empty() {
      prefs::tr("Cari Widget").to_string()
    } else {
      format!("{raw_query}|")
    };
    self.text(
      &content,
      &q,
      13.0 * d,
      400,
      fg,
      sb.0 + 28.0 * d,
      sb.1 + 7.0 * d,
      0,
      if query.is_empty() { 0.5 } else { 1.0 },
    );
    let mut cats = Vec::new();
    let mut y = 60.0 * d;
    let mut entries: Vec<(Option<Category>, &'static str, &'static str, [u8; 3])> =
      vec![(None, "Semua Widget", "\u{E71D}", [99, 99, 108])];
    for c in Category::ALL {
      let (gl, cc) = match c {
        Category::Clock => ("\u{E917}", ORANGE),
        Category::Calendar => ("\u{E787}", RED),
        Category::Battery => ("\u{E83F}", GREEN),
        Category::System => ("\u{E950}", BLUE),
      };
      entries.push((Some(c), c.title(), gl, cc));
    }
    for (c, title, gl, cc) in entries {
      let r: Rect = (10.0 * d, y, side - 20.0 * d, 34.0 * d);
      let Ok(row) = self.comp.CreateContainerVisual() else {
        continue;
      };
      let _ = content.Children().and_then(|k| k.InsertAtTop(&row));
      conts.push((r, row.clone(), motion::HOVER_ROW));
      if c == cat {
        let _ = self.rr(&row, r, 9.0 * d, crate::prefs::accent_c(220));
      }
      let ic: Rect = (r.0 + 8.0 * d, r.1 + 6.0 * d, 22.0 * d, 22.0 * d);
      let _ = self.rr(&row, ic, 6.0 * d, col(cc, 255));
      self.glyph(
        &row,
        gl,
        12.0 * d,
        [255, 255, 255],
        ic.0 + ic.2 / 2.0,
        ic.1 + ic.3 / 2.0,
        1.0,
      );
      self.text(
        &row,
        prefs::tr(title),
        13.0 * d,
        if c == cat { 600 } else { 500 },
        if c == cat { [255, 255, 255] } else { fg },
        r.0 + 40.0 * d,
        r.1 + 8.0 * d,
        0,
        1.0,
      );
      cats.push((c, r));
      y += 38.0 * d;
    }
    let x0 = side + 24.0 * d;
    let header = cat.map(|c| c.title()).unwrap_or("Semua Widget");
    self.text(&content, prefs::tr(header), 15.0 * d, 700, fg, x0, 16.0 * d, 0, 1.0);
    let list: Vec<Kind> = Kind::ALL
      .iter()
      .copied()
      .filter(|k| cat.map(|c| k.category() == c).unwrap_or(true))
      .filter(|k| {
        query.is_empty()
          || prefs::tr(k.title()).to_lowercase().contains(&query)
          || prefs::tr(k.category().title()).to_lowercase().contains(&query)
      })
      .collect();
    let (mut cx, mut cy) = (x0, 52.0 * d);
    let row_h = widgets::size_of(Kind::Clock, d).1 * PREVIEW + 62.0 * d;
    let right = gw - 20.0 * d;
    let mut cards = Vec::new();
    for k in list {
      let (w, h) = widgets::size_of(k, d);
      let (pw, ph) = (w * PREVIEW, h * PREVIEW);
      let cell_w = pw.max(130.0 * d);
      if cx + cell_w > right && cx > x0 {
        cx = x0;
        cy += row_h;
      }
      if cy + ph > gh - 60.0 * d {
        break;
      }
      let r: Rect = (cx + (cell_w - pw) / 2.0, cy, pw, ph);
      let Ok(card) = self.comp.CreateContainerVisual() else {
        continue;
      };
      let _ = content.Children().and_then(|kk| kk.InsertAtTop(&card));
      let Ok(inner) = self.comp.CreateContainerVisual() else {
        continue;
      };
      let _ = card.Children().and_then(|kk| kk.InsertAtTop(&inner));
      let _ = inner.SetOffset(Vector3 { X: r.0, Y: r.1, Z: 0.0 });
      let _ = inner.SetScale(Vector3 {
        X: PREVIEW,
        Y: PREVIEW,
        Z: 1.0,
      });
      let mut hands = self.draw(&inner, k, w, h, true);
      let n = time::local_now();
      for hd in hands.drain(..) {
        let (a, b, c) = widgets::hands(n.hour, n.minute, n.second);
        let _ = hd.v.SetRotationAngleInDegrees(match hd.which {
          0 => a,
          1 => b,
          _ => c,
        });
      }
      self.text(
        &card,
        prefs::tr(k.title()),
        12.0 * d,
        600,
        fg,
        cx + cell_w / 2.0,
        cy + ph + 8.0 * d,
        1,
        1.0,
      );
      self.text(
        &card,
        prefs::tr(k.desc()),
        10.5 * d,
        400,
        fg,
        cx + cell_w / 2.0,
        cy + ph + 26.0 * d,
        1,
        0.6,
      );
      conts.push((r, card, 1.06));
      cards.push((k, r));
      cx += cell_w + 22.0 * d;
    }
    let _ = self.rr(&content, (side, gh - 52.0 * d, gw - side, 1.0), 0.0, col(fg, 26));
    self.text(
      &content,
      prefs::tr("Seret widget ke desktop, atau klik untuk menambahkannya."),
      12.0 * d,
      400,
      fg,
      x0,
      gh - 34.0 * d,
      0,
      0.6,
    );
    let done: Rect = (gw - 104.0 * d, gh - 42.0 * d, 86.0 * d, 30.0 * d);
    if let Ok(dc) = self.comp.CreateContainerVisual() {
      let _ = content.Children().and_then(|k| k.InsertAtTop(&dc));
      let _ = self.rr(&dc, done, 15.0 * d, crate::prefs::accent_c(255));
      self.text(
        &dc,
        prefs::tr("Selesai"),
        13.0 * d,
        600,
        [255, 255, 255],
        done.0 + done.2 / 2.0,
        done.1 + 6.0 * d,
        1,
        1.0,
      );
      conts.push((done, dc, motion::HOVER_BUTTON));
    }
    if let Some(g) = self.gallery.as_mut() {
      g.cards = cards;
      g.cats = cats;
      g.done = done;
      g.conts = conts;
      g.hover = None;
    }
  }

  pub(super) fn gallery_hover(&mut self, i: Option<usize>) {
    let comp = self.comp.clone();
    let Some(g) = self.gallery.as_mut() else {
      return;
    };
    if g.hover == i {
      return;
    }
    for (k, on) in [(g.hover, false), (i, true)] {
      if let Some((r, c, to)) = k.and_then(|k| g.conts.get(k)) {
        let _ = motion::hover_pop(
          &comp,
          c,
          Vector3 {
            X: r.0 + r.2 / 2.0,
            Y: r.1 + r.3 / 2.0,
            Z: 0.0,
          },
          if on { *to } else { 1.0 },
        );
      }
    }
    g.hover = i;
  }
}
