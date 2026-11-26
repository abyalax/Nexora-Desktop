use super::spring;
use super::*;

impl Spot {
  pub(super) fn render(&mut self) {
    self.render_with(false);
  }

  pub(super) fn render_with(&mut self, intro: bool) {
    if self.morphing {
      return;
    }
    let content = self.popup.content.clone();
    if let Ok(k) = content.Children() {
      let _ = k.RemoveAll();
    }
    self.hits.clear();
    self.circles.clear();
    if self.compact() {
      self.render_compact(&content);
    } else {
      self.render_panel(&content, intro);
    }
  }

  pub(super) fn render_compact(&mut self, content: &ContainerVisual) {
    let (g, d) = (self.geo, self.dpi);
    let fg = self.look().fg;
    let _ = content.SetOpacity(1.0);
    let t = if self.buttons {
      self.hover_rect()
    } else {
      self.idle_rect()
    };
    let was_panel = self.cur.3 > g.h + 0.5;
    let old_x = self.cur.0;
    self.place(t, Some(if was_panel { (0.95, 70) } else { (0.82, 75) }));
    let dx = old_x - t.0;
    if dx.abs() > 0.5 && !was_panel {
      let _ = content.SetOffset(Vector3 { X: dx, Y: 0.0, Z: 0.0 });
      if let Ok(o) = content.cast::<CompositionObject>() {
        spring(&self.comp, &o, "Offset.X", 0.0, 0.82, 75);
      }
    } else if was_panel {
      let _ = content.SetOffset(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 });
    }
    let cy = g.h / 2.0;
    self.sym(content, Sym::Search, t.0 + 28.0 * d, cy, 21.0 * d, fg, 0.75);
    let label = match self.hover_mode {
      Some(m) => prefs::tr(m.title()).to_string(),
      None => prefs::tr("Quick Search").to_string(),
    };
    let mut avail = t.2 - 72.0 * d;
    if let Some(m) = self.hover_mode {
      let key = format!("Ctrl {}", m.digit());
      let kpx = (12.0 * d).round() as i32;
      let kw = self.text_w(&key, kpx, 500) + 14.0 * d;
      let r: Rect = (t.0 + t.2 - kw - 16.0 * d, cy - 11.0 * d, kw, 22.0 * d);
      self.rect(
        content,
        r,
        6.0 * d,
        Color {
          A: 30,
          R: fg[0],
          G: fg[1],
          B: fg[2],
        },
      );
      self.text(content, &key, r.0 + 7.0 * d, cy, kpx, 500, fg, 0.65, kw);
      avail -= kw + 12.0 * d;
    }
    self.text(
      content,
      &label,
      t.0 + 52.0 * d,
      cy,
      (21.0 * d).round() as i32,
      400,
      fg,
      0.5,
      avail,
    );

    let side = self.side.clone();
    if self.buttons {
      if let Ok(k) = side.Children() {
        let _ = k.RemoveAll();
      }
      let _ = side.SetOpacity(1.0);
      self.side_fading = false;
      for (i, m) in [Mode::Apps, Mode::Files, Mode::Actions, Mode::Clipboard]
        .into_iter()
        .enumerate()
      {
        let r: Rect = (g.cap + g.gap + i as f32 * (g.btn + g.gap), 0.0, g.btn, g.btn);
        let Ok(cv) = self.comp.CreateContainerVisual() else {
          continue;
        };
        let _ = side.Children().and_then(|k| k.InsertAtTop(&cv));
        if let Some(gp) = self.glass_circle(&cv, r) {
          self.circles.push(gp);
        }
        if self.hover_mode == Some(m) {
          self.rect(
            &cv,
            r,
            g.btn / 2.0,
            Color {
              A: 36,
              R: fg[0],
              G: fg[1],
              B: fg[2],
            },
          );
        }
        self.mode_icon(&cv, m, r.0 + g.btn / 2.0, r.1 + g.btn / 2.0, 21.0 * d, fg, 0.9);
        if self.buttons_intro {
          let center = Vector3 {
            X: r.0 + g.btn / 2.0,
            Y: r.1 + g.btn / 2.0,
            Z: 0.0,
          };
          let _ = crate::motion::pop_in(&self.comp, &cv, center, 0.55, 1.05, 20 + i as u32 * 28, 300);
        }
        self.hits.push((r, HitKind::Mode(m)));
      }
      self.buttons_intro = false;
    } else if !self.side_fading && side.Children().map(|k| k.Count().unwrap_or(0)).unwrap_or(0) > 0 {
      let _ = crate::motion::scalar(&self.comp, &side, "Opacity", 1.0, 0.0, 130);
      let _ = side.SetOpacity(0.0);
      self.side_fading = true;
    }
  }

  pub(super) fn render_panel(&mut self, content: &ContainerVisual, intro: bool) {
    let (g, d) = (self.geo, self.dpi);
    let fg = self.look().fg;
    let was_compact = self.cur.3 <= g.h + 0.5;
    let t = self.panel_rect();
    self.place(
      t,
      Some(if intro {
        (0.9, 90)
      } else if was_compact {
        (0.92, 80)
      } else {
        (1.0, 55)
      }),
    );
    let _ = content.SetOffset(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 });
    if intro {
      let go = || -> windows_core::Result<()> {
        let e = crate::motion::ease(&self.comp)?;
        let a = self.comp.CreateScalarKeyFrameAnimation()?;
        a.InsertKeyFrame(0.0, 0.0)?;
        a.InsertKeyFrame(0.35, 0.0)?;
        a.InsertKeyFrameWithEasingFunction(1.0, 1.0, &e)?;
        a.SetDuration(windows::Foundation::TimeSpan { Duration: 320 * 10_000 })?;
        content
          .cast::<CompositionObject>()?
          .StartAnimation(&HSTRING::from("Opacity"), &a)
      };
      let _ = go();
    }
    let _ = content.SetOpacity(1.0);
    if let Ok(k) = self.side.Children() {
      let _ = k.RemoveAll();
    }
    let Ok(body) = self.comp.CreateContainerVisual() else {
      return;
    };
    let _ = body.SetOffset(Vector3 { X: t.0, Y: 0.0, Z: 0.0 });
    let _ = content.Children().and_then(|k| k.InsertAtTop(&body));
    let hits0 = self.hits.len();
    let (w, ph) = (t.2, t.3);

    let cy = g.h / 2.0;
    self.mode_icon(&body, self.mode, 28.0 * d, cy, 21.0 * d, fg, 0.8);
    let px = (21.0 * d).round() as i32;
    let tx = 52.0 * d;
    let right_pad = 70.0 * d;
    if self.query.is_empty() {
      self.text(
        &body,
        prefs::tr(self.mode.title()),
        tx,
        cy,
        px,
        400,
        fg,
        0.5,
        w - tx - right_pad,
      );
    } else {
      let q = self.query.clone();
      let qw = self.text(&body, &q, tx, cy, px, 400, fg, 1.0, w - tx - right_pad - 90.0 * d);
      if self.mode == Mode::Search && !self.rows.is_empty() {
        let open = format!("\u{2014} {}", prefs::tr("Buka"));
        self.text(
          &body,
          &open,
          tx + qw + 8.0 * d,
          cy,
          (16.0 * d).round() as i32,
          400,
          fg,
          0.42,
          160.0 * d,
        );
      }
    }
    let has_more = matches!(self.mode, Mode::Apps | Mode::Files | Mode::Clipboard);
    if self.mode == Mode::Search {
      if let Some(row) = self.rows.get(self.sel).cloned() {
        if matches!(row.kind, RKind::App | RKind::File) {
          if let Some(s) = self.icon_surface(&row.path, (64.0 * d) as i32) {
            self.sprite(&body, &s, w - 52.0 * d, cy - 16.0 * d, 32.0 * d, 32.0 * d, 1.0);
          }
        }
      }
    } else if has_more {
      let r: Rect = (w - 46.0 * d, cy - 14.0 * d, 28.0 * d, 28.0 * d);
      self.rect(
        &body,
        r,
        14.0 * d,
        Color {
          A: if self.more_open { 64 } else { 26 },
          R: fg[0],
          G: fg[1],
          B: fg[2],
        },
      );
      self.glyph(&body, "\u{E712}", r.0 + 14.0 * d, cy, 13.0 * d, fg, 0.85);
      self.hits.push((r, HitKind::More));
    }
    if !(self.mode == Mode::Search && self.rows.is_empty()) {
      self.rect(
        &body,
        (16.0 * d, g.h - 1.0, w - 32.0 * d, 1.0),
        0.0,
        Color {
          A: 30,
          R: fg[0],
          G: fg[1],
          B: fg[2],
        },
      );
    }
    let mut top = g.h + 2.0 * d;

    if matches!(self.mode, Mode::Search | Mode::Files) {
      let chips_h = 42.0 * d;
      let mut x = 20.0 * d;
      let cy2 = top + chips_h / 2.0;
      let cpx = (13.0 * d).round() as i32;
      let items: Vec<(String, HitKind, bool)> = if self.mode == Mode::Search {
        [
          (prefs::tr("Aplikasi"), Filter::Apps),
          (prefs::tr("Berkas Saya"), Filter::Files),
          (prefs::tr("Aksi"), Filter::Actions),
        ]
        .into_iter()
        .map(|(l, f)| (l.to_string(), HitKind::Filter(f), self.filter == Some(f)))
        .collect()
      } else {
        FTYPES
          .iter()
          .enumerate()
          .map(|(i, (l, _))| (prefs::tr(l).to_string(), HitKind::FType(i), self.ftype == Some(i)))
          .collect()
      };
      for (label, f, active) in items {
        let cw = self.text_w(&label, cpx, 500) + 24.0 * d;
        let r: Rect = (x, cy2 - 13.0 * d, cw, 26.0 * d);
        let col = if active {
          crate::prefs::accent_c(90)
        } else {
          Color {
            A: 30,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          }
        };
        self.rect(&body, r, 13.0 * d, col);
        self.text(
          &body,
          &label,
          r.0 + 12.0 * d,
          cy2,
          cpx,
          500,
          fg,
          if active { 1.0 } else { 0.78 },
          cw,
        );
        self.hits.push((r, f));
        x += cw + 8.0 * d;
      }
      top += chips_h;
    }

    let Ok(area) = self.comp.CreateContainerVisual() else {
      return;
    };
    let _ = area.SetSize(Vector2 { X: w, Y: ph });
    if let Ok(ac) = self.comp.CreateInsetClip() {
      let _ = ac.SetTopInset(top);
      let _ = area.SetClip(&ac);
    }
    let _ = body.Children().and_then(|k| k.InsertAtTop(&area));
    let empty_q = self.query.trim().is_empty();
    match self.mode {
      Mode::Apps if empty_q => self.draw_sections(&area, w, top, ph, fg, false),
      Mode::Apps => self.draw_grid(&area, w, top, fg),
      Mode::Files if empty_q => self.draw_sections(&area, w, top, ph, fg, true),
      _ => self.draw_rows(&area, w, top, 54.0 * d, ph, fg),
    }

    if self.more_open && has_more {
      let label = match self.mode {
        Mode::Clipboard => prefs::tr("Hapus Riwayat"),
        Mode::Apps => prefs::tr(if self.expanded.iter().all(|e| *e) {
          "Ringkas Semua"
        } else {
          "Tampilkan Semua"
        }),
        _ => prefs::tr("Buka Folder Terbaru"),
      };
      let mpx = (13.0 * d).round() as i32;
      let mw = self.text_w(label, mpx, 500) + 28.0 * d;
      let r: Rect = (w - mw - 18.0 * d, g.h - 4.0 * d, mw, 32.0 * d);
      if let Some(gp) = self.glass_circle(&body, r) {
        let _ = gp.set_radius(10.0 * d);
        self.circles.push(gp);
      }
      self.text(&body, label, r.0 + 14.0 * d, r.1 + r.3 / 2.0, mpx, 500, fg, 0.95, mw);
      self.hits.insert(hits0, (r, HitKind::MoreItem));
    }

    for h in &mut self.hits[hits0..] {
      h.0.0 += t.0;
    }
  }
}
