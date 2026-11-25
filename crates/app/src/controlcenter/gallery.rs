use super::*;

impl Cc {
  pub(super) fn open_gallery(&mut self) {
    let Some(p) = self.panel.as_ref() else { return };
    let dpi = p.dpi;
    let base = GlassParams::dark((10.0, 10.0), 26.0 * dpi);
    let look = prefs::look(prefs::Role::Dark);
    let Ok(mut popup) = Popup::new_with(
      &self.comp,
      w!("NexoraCcGallery"),
      Some(gallery_proc),
      WINDOW_EX_STYLE(0),
      prefs::apply(base, &look),
      true,
    ) else {
      return;
    };
    let (gw, gh) = (620.0 * dpi, 560.0 * dpi);
    let rect = p.popup.rect;
    let x = (rect.0 as f32 + pad() - gw - 16.0 * dpi).max(10.0 * dpi);
    let _ = popup.set_rect(x as i32, rect.1 + pad() as i32, gw as i32, gh as i32);
    let _ = popup.panel.set_radius(26.0 * dpi);
    self.gallery = Some(Gallery {
      popup,
      dpi,
      cat: None,
      query: String::new(),
      cards: vec![],
      cats: vec![],
      done: (0.0, 0.0, 0.0, 0.0),
      scroll: 0.0,
      content_h: 0.0,
      panels: vec![],
      conts: vec![],
      hover: None,
      tracking: false,
    });
    self.refresh_gallery();
    if let Some(g) = &self.gallery {
      g.popup.show();
      let _ = motion::pop_in(
        &self.comp,
        g.popup.root(),
        Vector3 {
          X: g.popup.rect.2 as f32,
          Y: 0.0,
          Z: 0.0,
        },
        0.82,
        1.02,
        0,
        340,
      );
    }
  }
  pub(super) fn refresh_gallery(&mut self) {
    let Some(g) = self.gallery.as_ref() else {
      return;
    };
    let (dpi, cat, query, scroll) = (g.dpi, g.cat, g.query.to_lowercase(), g.scroll);
    let content = g.popup.content.clone();
    let (gw, gh) = (g.popup.rect.2 as f32, g.popup.rect.3 as f32);
    if let Ok(k) = content.Children() {
      let _ = k.RemoveAll();
    }
    let side = 190.0 * dpi;
    let mut cats = Vec::new();
    let mut conts: Vec<(Rect, ContainerVisual, f32)> = Vec::new();
    let sb: Rect = (16.0 * dpi, 16.0 * dpi, side - 8.0 * dpi, 36.0 * dpi);
    let _ = self.rrect(
      &content,
      sb,
      10.0 * dpi,
      Color {
        A: 60,
        R: 255,
        G: 255,
        B: 255,
      },
    );
    let q = if query.is_empty() {
      prefs::tr("Cari Kontrol").to_string()
    } else {
      g.query.clone()
    };
    self.text(
      &content,
      &q,
      (13.0 * dpi) as i32,
      400,
      WHITE,
      sb.0 + 12.0 * dpi,
      sb.1 + 9.0 * dpi,
      sb.2 - 20.0 * dpi,
      if query.is_empty() { 0.55 } else { 1.0 },
    );
    let mut y = 66.0 * dpi;
    let mut entries: Vec<(Option<Category>, &str)> = vec![(None, "Semua Kontrol")];
    entries.extend(Category::ALL.iter().map(|c| (Some(*c), c.title())));
    for (c, title) in entries {
      let r: Rect = (12.0 * dpi, y, side, 36.0 * dpi);
      let Ok(row) = self.comp.CreateContainerVisual() else {
        continue;
      };
      let _ = content.Children().and_then(|k| k.InsertAtTop(&row));
      conts.push((r, row.clone(), motion::HOVER_ROW));
      let content = row;
      if c == cat {
        let _ = self.rrect(
          &content,
          r,
          10.0 * dpi,
          Color {
            A: 70,
            R: 255,
            G: 255,
            B: 255,
          },
        );
      }
      let (gl, col): (&str, [u8; 3]) = match c {
        None => ("\u{E71D}", [99, 99, 108]),
        Some(Category::Connectivity) => ("\u{E701}", [10, 132, 255]),
        Some(Category::Display) => ("\u{E706}", [10, 132, 255]),
        Some(Category::Sound) => ("\u{E995}", [255, 45, 85]),
        Some(Category::Media) => ("\u{E8D6}", [255, 69, 58]),
        Some(Category::Battery) => ("\u{E83F}", [52, 199, 89]),
        Some(Category::Utilities) => ("\u{E713}", [142, 142, 147]),
      };
      let ic: Rect = (r.0 + 10.0 * dpi, r.1 + 7.0 * dpi, 22.0 * dpi, 22.0 * dpi);
      let _ = self.rrect(
        &content,
        ic,
        6.0 * dpi,
        Color {
          A: 255,
          R: col[0],
          G: col[1],
          B: col[2],
        },
      );
      self.glyph(
        &content,
        gl,
        (12.0 * dpi) as i32,
        WHITE,
        ic.0 + ic.2 / 2.0,
        ic.1 + ic.3 / 2.0,
        1.0,
      );
      self.text(
        &content,
        prefs::tr(title),
        (13.0 * dpi) as i32,
        500,
        WHITE,
        r.0 + 42.0 * dpi,
        r.1 + 9.0 * dpi,
        r.2 - 48.0 * dpi,
        1.0,
      );
      cats.push((c, r));
      y += 38.0 * dpi;
    }
    let mut cards = Vec::new();
    let list: Vec<Kind> = controls::ALL
      .iter()
      .copied()
      .filter(|k| cat.map(|c| k.category() == c).unwrap_or(true))
      .filter(|k| query.is_empty() || prefs::tr(k.title()).to_lowercase().contains(&query))
      .collect();
    let (cx0, mut cy) = (side + 36.0 * dpi, 20.0 * dpi - scroll);
    let (cw, ch, cg) = (130.0 * dpi, 112.0 * dpi, 14.0 * dpi);
    let total_y0 = 20.0 * dpi;
    for c in Category::ALL {
      if cat.map(|x| x != c).unwrap_or(false) {
        continue;
      }
      let ks: Vec<Kind> = list.iter().copied().filter(|k| k.category() == c).collect();
      if ks.is_empty() {
        continue;
      }
      if cy + 24.0 * dpi > 0.0 && cy < gh - 60.0 * dpi {
        self.text(
          &content,
          prefs::tr(c.title()),
          (13.0 * dpi) as i32,
          600,
          WHITE,
          cx0,
          cy,
          3.0 * cw,
          0.85,
        );
      }
      cy += 28.0 * dpi;
      for (i, k) in ks.iter().enumerate() {
        let (col, row) = (i % 3, i / 3);
        let r: Rect = (cx0 + col as f32 * (cw + cg), cy + row as f32 * (ch + cg), cw, ch);
        if r.1 + r.3 < 0.0 || r.1 > gh - 60.0 * dpi {
          continue;
        }
        let present = self.layout.contains(k);
        let Ok(card) = self.comp.CreateContainerVisual() else {
          continue;
        };
        let _ = content.Children().and_then(|k| k.InsertAtTop(&card));
        conts.push((r, card.clone(), 1.06));
        let content = card;
        let _ = self.rrect(
          &content,
          r,
          20.0 * dpi,
          Color {
            A: if present { 30 } else { 85 },
            R: 255,
            G: 255,
            B: 255,
          },
        );
        let disc: Rect = (r.0 + r.2 / 2.0 - 22.0 * dpi, r.1 + 18.0 * dpi, 44.0 * dpi, 44.0 * dpi);
        let _ = self.rrect(
          &content,
          disc,
          22.0 * dpi,
          Color {
            A: if present { 60 } else { 150 },
            R: 0,
            G: 0,
            B: 0,
          },
        );
        self.glyph(
          &content,
          glyph_of(*k),
          (20.0 * dpi) as i32,
          WHITE,
          disc.0 + 22.0 * dpi,
          disc.1 + 22.0 * dpi,
          if present { 0.5 } else { 1.0 },
        );
        self.text(
          &content,
          prefs::tr(k.title()),
          (12.0 * dpi) as i32,
          500,
          WHITE,
          r.0 + 10.0 * dpi,
          r.1 + r.3 - 30.0 * dpi,
          r.2 - 20.0 * dpi,
          if present { 0.5 } else { 1.0 },
        );
        cards.push((*k, r));
      }
      cy += ks.len().div_ceil(3) as f32 * (ch + cg);
      if r_sep_visible(cy, gh, dpi) {
        let _ = self.rrect(
          &content,
          (cx0, cy - cg / 2.0, 3.0 * cw + 2.0 * cg, 1.0),
          0.5,
          Color {
            A: 40,
            R: 255,
            G: 255,
            B: 255,
          },
        );
      }
      cy += 14.0 * dpi;
    }
    let content_h = cy + scroll - total_y0;
    if let Some(g) = self.gallery.as_mut() {
      g.content_h = content_h;
    }
    self.text(
      &content,
      prefs::tr("Klik kontrol untuk menambahkannya ke Quick Panel."),
      (12.0 * dpi) as i32,
      400,
      WHITE,
      side + 36.0 * dpi,
      gh - 34.0 * dpi,
      gw - side - 180.0 * dpi,
      0.55,
    );
    let done: Rect = (gw - 100.0 * dpi, gh - 46.0 * dpi, 84.0 * dpi, 32.0 * dpi);
    if let Ok(dc) = self.comp.CreateContainerVisual() {
      let _ = content.Children().and_then(|k| k.InsertAtTop(&dc));
      let _ = self.rrect(&dc, done, 16.0 * dpi, prefs::accent_c(255));
      self.text(
        &dc,
        prefs::tr("Selesai"),
        (13.0 * dpi) as i32,
        600,
        WHITE,
        done.0 + 18.0 * dpi,
        done.1 + 7.0 * dpi,
        70.0 * dpi,
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
  pub(super) fn update_values(&mut self) {
    let Some(p) = self.panel.as_ref() else { return };
    for m in &p.mods {
      if let (Some(t), Some(f)) = (m.track, &m.fill) {
        let frac = if m.placed.kind == Kind::Display {
          brightness::latest().map(|v| v as f32 / 100.0).unwrap_or(0.5)
        } else {
          audio::latest().level
        };
        if m.vertical {
          place_slider_v(f, &m.thumb, t, frac, p.dpi);
        } else {
          place_slider(f, &m.thumb, t, frac, p.dpi);
        }
      }
    }
  }
}
