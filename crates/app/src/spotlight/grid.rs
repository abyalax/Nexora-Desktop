use super::*;

impl Spot {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn draw_sections(&mut self, area: &ContainerVisual, w: f32, top: f32, ph: f32, fg: [u8; 3], thumbs: bool) {
    let d = self.dpi;
    let cw = (w - 32.0 * d) / COLS as f32;
    let (icon, ch) = if thumbs {
      (80.0 * d, 124.0 * d)
    } else {
      (56.0 * d, 98.0 * d)
    };
    let hh = 34.0 * d;
    self.view = (top, ph);
    self.cell_y.clear();
    if self.rows.is_empty() {
      let msg = if thumbs {
        prefs::tr("Belum ada berkas terbaru.")
      } else {
        prefs::tr("Tidak ada aplikasi")
      };
      self.text(
        area,
        msg,
        24.0 * d,
        top + 40.0 * d,
        (15.0 * d).round() as i32,
        400,
        fg,
        0.6,
        w - 40.0 * d,
      );
      self.content_h = 0.0;
      return;
    }
    let vis = |y0: f32, h: f32| y0 + h > top && y0 < ph;
    let mut y = top + 4.0 * d - self.pscroll;
    let secs = self.sections.clone();
    for (si, (title, range)) in secs.iter().enumerate() {
      let expanded = self.expanded.get(si).copied().unwrap_or(false);
      let total = range.len();
      let shown = if expanded { total } else { total.min(COLS) };
      if vis(y, hh) {
        self.text(
          area,
          title,
          18.0 * d,
          y + hh / 2.0,
          (13.0 * d).round() as i32,
          600,
          fg,
          0.62,
          w * 0.6,
        );
        if total > COLS && !thumbs {
          let label = prefs::tr(if expanded { "Lebih Sedikit" } else { "Lihat Semua" });
          let lpx = (12.5 * d).round() as i32;
          let lw = self.text_w(label, lpx, 500);
          let r: Rect = (w - lw - 24.0 * d, y + 4.0 * d, lw + 8.0 * d, hh - 8.0 * d);
          self.text(
            area,
            label,
            r.0 + 4.0 * d,
            y + hh / 2.0,
            lpx,
            500,
            fg,
            0.55,
            lw + 4.0 * d,
          );
          self.hits.push((r, HitKind::ShowMore(si)));
        }
      }
      y += hh;
      for k in 0..shown {
        let idx = range.start + k;
        let (r, c) = (k / COLS, k % COLS);
        let cx = 16.0 * d + c as f32 * cw + cw / 2.0;
        let y0 = y + r as f32 * ch;
        self.cell_y.push((idx, y0 + self.pscroll, ch));
        if !vis(y0, ch) {
          continue;
        }
        let row = self.rows[idx].clone();
        let sel_c;
        let area = if idx == self.sel {
          sel_c = self.sel_box(
            area,
            &row,
            Vector3 {
              X: cx,
              Y: y0 + ch / 2.0,
              Z: 0.0,
            },
            cw / 2.0,
            12.0 * d,
          );
          &sel_c
        } else {
          area
        };
        if idx == self.sel {
          self.rect(
            area,
            (cx - cw / 2.0 + 4.0 * d, y0, cw - 8.0 * d, ch - 4.0 * d),
            14.0 * d,
            Color {
              A: 26,
              R: fg[0],
              G: fg[1],
              B: fg[2],
            },
          );
        }
        let iy = y0 + 6.0 * d;
        if thumbs {
          if let Some((s, tw, th)) = self.thumb_for(&row.path, icon) {
            self.sprite(area, &s, cx - tw / 2.0, iy + (icon - th) / 2.0, tw, th, 1.0);
          } else if let Some(s) = self.icon_surface(&row.path, (icon * 1.2) as i32) {
            self.sprite(
              area,
              &s,
              cx - icon * 0.35,
              iy + icon * 0.15,
              icon * 0.7,
              icon * 0.7,
              1.0,
            );
          }
        } else if let Some(s) = self.icon_surface(&row.path, (icon * 1.6) as i32) {
          self.sprite(area, &s, cx - icon / 2.0, iy, icon, icon, 1.0);
        } else {
          self.rect(
            area,
            (cx - icon / 2.0, iy, icon, icon),
            icon * 0.23,
            Color {
              A: 60,
              R: fg[0],
              G: fg[1],
              B: fg[2],
            },
          );
        }
        let lpx = (12.0 * d).round() as i32;
        let lw = self.text_w(&row.title, lpx, 400).min(cw - 10.0 * d);
        self.text(
          area,
          &row.title,
          cx - lw / 2.0,
          iy + icon + 13.0 * d,
          lpx,
          400,
          fg,
          0.92,
          cw - 10.0 * d,
        );
        self.hits.push(((cx - cw / 2.0, y0, cw, ch), HitKind::Cell(idx)));
      }
      y += shown.div_ceil(COLS) as f32 * ch + 6.0 * d;
      if si + 1 < secs.len() && vis(y, 1.0) {
        self.rect(
          area,
          (16.0 * d, y, w - 32.0 * d, 1.0),
          0.0,
          Color {
            A: 26,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
      }
      y += 8.0 * d;
    }
    self.content_h = y + self.pscroll - top;
  }

  pub(super) fn cell_h(&self) -> f32 {
    112.0 * self.dpi
  }

  pub(super) fn draw_grid(&mut self, content: &ContainerVisual, w: f32, top: f32, fg: [u8; 3]) {
    let d = self.dpi;
    let (cw, ch) = ((w - 40.0 * d) / COLS as f32, self.cell_h());
    let icon = (cw * 0.5).min(72.0 * d);
    let per = COLS * GRID_ROWS;
    let start = self.scroll * COLS;
    let n = self.rows.len();
    if n == 0 {
      self.text(
        content,
        prefs::tr("Tidak ada aplikasi"),
        w / 2.0 - 60.0 * d,
        top + 60.0 * d,
        (15.0 * d).round() as i32,
        400,
        fg,
        0.6,
        w,
      );
      return;
    }
    for i in 0..per {
      let idx = start + i;
      if idx >= n {
        break;
      }
      let (r, c) = (i / COLS, i % COLS);
      let cx = 20.0 * d + c as f32 * cw + cw / 2.0;
      let cy0 = top + 6.0 * d + r as f32 * ch;
      let row = self.rows[idx].clone();
      let sel_c;
      let content = if idx == self.sel {
        sel_c = self.sel_box(
          content,
          &row,
          Vector3 {
            X: cx,
            Y: cy0 + ch / 2.0,
            Z: 0.0,
          },
          cw / 2.0,
          16.0 * d,
        );
        &sel_c
      } else {
        content
      };
      if idx == self.sel {
        self.rect(
          content,
          (cx - cw / 2.0 + 4.0 * d, cy0, cw - 8.0 * d, ch - 4.0 * d),
          14.0 * d,
          Color {
            A: 26,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
      }
      if let Some(s) = self.icon_surface(&row.path, (icon * 1.6) as i32) {
        self.sprite(content, &s, cx - icon / 2.0, cy0 + 8.0 * d, icon, icon, 1.0);
      } else {
        self.rect(
          content,
          (cx - icon / 2.0, cy0 + 8.0 * d, icon, icon),
          icon * 0.23,
          Color {
            A: 60,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
      }
      let lw = self
        .text_w(&row.title, (13.0 * d).round() as i32, 400)
        .min(cw - 10.0 * d);
      self.text(
        content,
        &row.title,
        cx - lw / 2.0,
        cy0 + 8.0 * d + icon + 14.0 * d,
        (13.0 * d).round() as i32,
        400,
        fg,
        0.95,
        cw - 10.0 * d,
      );
      self.hits.push(((cx - cw / 2.0, cy0, cw, ch), HitKind::Cell(idx)));
    }
  }

  pub(super) fn sel_box(
    &mut self,
    parent: &ContainerVisual,
    row: &Row,
    center: Vector3,
    half: f32,
    room: f32,
  ) -> ContainerVisual {
    let k = motion::fit_pop(motion::HOVER_ROW, half, room);
    let Ok(c) = self.comp.CreateContainerVisual() else {
      return parent.clone();
    };
    let _ = parent.Children().and_then(|k| k.InsertAtTop(&c));
    let key = format!("{}|{}", row.title, row.path);
    if self.popped.as_deref() == Some(key.as_str()) {
      let _ = c.SetCenterPoint(center);
      let _ = c.SetScale(Vector3 { X: k, Y: k, Z: 1.0 });
    } else {
      let _ = motion::hover_pop(&self.comp, &c, center, k);
      self.popped = Some(key);
    }
    c
  }
}
