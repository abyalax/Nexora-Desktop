use super::action_glyph;
use super::*;

impl Spot {
  #[allow(clippy::too_many_arguments)]
  pub(super) fn draw_rows(&mut self, area: &ContainerVisual, w: f32, top: f32, row_h: f32, bottom: f32, fg: [u8; 3]) {
    let d = self.dpi;
    if self.rows.is_empty() {
      if self.mode != Mode::Search {
        let msg = match self.mode {
          Mode::Clipboard => prefs::tr("Belum ada yang disalin dalam 8 jam terakhir."),
          _ => prefs::tr("Tidak ada hasil"),
        };
        let hint = if self.mode == Mode::Files && self.query.trim().chars().count() < 2 {
          prefs::tr("Ketik untuk mencari berkas.")
        } else {
          msg
        };
        self.text(
          area,
          hint,
          26.0 * d,
          top + row_h / 2.0,
          (15.0 * d).round() as i32,
          400,
          fg,
          0.6,
          w - 40.0 * d,
        );
      }
      return;
    }
    let first = self.scroll.min(self.rows.len().saturating_sub(1));
    let headers = self.headers.clone();
    let mut ry = top + 4.0 * d;
    for i in first..self.rows.len() {
      if ry > bottom {
        break;
      }
      if let Some((_, title)) = headers.iter().find(|(at, _)| *at == i) {
        self.text(
          area,
          title,
          20.0 * d,
          ry + 15.0 * d,
          (13.0 * d).round() as i32,
          600,
          fg,
          0.6,
          w * 0.6,
        );
        ry += 30.0 * d;
      }
      let row = self.rows[i].clone();
      let sel_c;
      let area = if i == self.sel {
        sel_c = self.sel_box(
          area,
          &row,
          Vector3 {
            X: w / 2.0,
            Y: ry + row_h / 2.0,
            Z: 0.0,
          },
          w / 2.0 - 8.0 * d,
          8.0 * d,
        );
        &sel_c
      } else {
        area
      };
      if i == self.sel {
        self.rect(
          area,
          (8.0 * d, ry, w - 16.0 * d, row_h - 2.0 * d),
          12.0 * d,
          Color {
            A: 28,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
      }
      let ic = 32.0 * d;
      let (icx, icy) = (22.0 * d + ic / 2.0, ry + (row_h - 2.0 * d) / 2.0);
      match &row.kind {
        RKind::App | RKind::File => {
          if let Some(s) = self.icon_surface(&row.path, (ic * 2.0) as i32) {
            self.sprite(area, &s, icx - ic / 2.0, icy - ic / 2.0, ic, ic, 1.0);
          }
        }
        RKind::Calc => {
          self.rect(
            area,
            (icx - ic / 2.0, icy - ic / 2.0, ic, ic),
            8.0 * d,
            Color {
              A: 255,
              R: 255,
              G: 149,
              B: 0,
            },
          );
          self.glyph(area, "\u{E8EF}", icx, icy, 17.0 * d, [255, 255, 255], 1.0);
        }
        RKind::Web => {
          self.rect(
            area,
            (icx - ic / 2.0, icy - ic / 2.0, ic, ic),
            8.0 * d,
            crate::prefs::accent_c(255),
          );
          self.glyph(area, "\u{E774}", icx, icy, 17.0 * d, [255, 255, 255], 1.0);
        }
        RKind::Action(_) => {
          let (g, col) = action_glyph(&row.path);
          self.rect(
            area,
            (icx - ic / 2.0, icy - ic / 2.0, ic, ic),
            8.0 * d,
            Color {
              A: 255,
              R: col[0],
              G: col[1],
              B: col[2],
            },
          );
          self.glyph(area, g, icx, icy, 17.0 * d, [255, 255, 255], 1.0);
        }
        RKind::Clip => {
          self.rect(
            area,
            (icx - ic * 0.36, icy - ic / 2.0, ic * 0.72, ic),
            4.0 * d,
            Color {
              A: 240,
              R: 250,
              G: 250,
              B: 252,
            },
          );
          self.glyph(area, "\u{E8A5}", icx, icy, 14.0 * d, [120, 120, 128], 1.0);
        }
      }
      let tx = 22.0 * d + ic + 14.0 * d;
      let max_w = w
        - tx
        - if matches!(row.kind, RKind::Clip) {
          64.0 * d
        } else {
          150.0 * d
        };
      self.text(
        area,
        &row.title,
        tx,
        icy - 9.0 * d,
        (15.0 * d).round() as i32,
        400,
        fg,
        1.0,
        max_w,
      );
      if !row.sub.is_empty() {
        self.text(
          area,
          &row.sub,
          tx,
          icy + 11.0 * d,
          (12.0 * d).round() as i32,
          400,
          fg,
          0.55,
          max_w,
        );
      }
      if let RKind::Action(ai) = row.kind {
        let key = model::key_of(&ACTIONS[ai], &self.quick).to_string();
        let editing = self.editing == Some(ai);
        let label = if editing {
          format!("{}|", self.edit_buf)
        } else if key.is_empty() {
          prefs::tr("Tambah kunci cepat").to_string()
        } else {
          key
        };
        let cpx = (12.0 * d).round() as i32;
        let cw = self.text_w(&label, cpx, 500) + 18.0 * d;
        let r: Rect = (w - cw - 24.0 * d, icy - 11.0 * d, cw, 22.0 * d);
        self.rect(
          area,
          r,
          6.0 * d,
          Color {
            A: if editing { 80 } else { 36 },
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
        self.text(area, &label, r.0 + 9.0 * d, icy, cpx, 500, fg, 0.75, cw);
      }
      if matches!(row.kind, RKind::Clip) {
        let r: Rect = (w - 48.0 * d, icy - 13.0 * d, 26.0 * d, 26.0 * d);
        self.rect(
          area,
          r,
          13.0 * d,
          Color {
            A: 30,
            R: fg[0],
            G: fg[1],
            B: fg[2],
          },
        );
        self.glyph(area, "\u{E8C8}", r.0 + 13.0 * d, icy, 12.0 * d, fg, 0.8);
        self.hits.push((r, HitKind::CopyRow(i)));
      }
      self.hits.push(((0.0, ry, w, row_h), HitKind::Row(i)));
      ry += row_h;
    }
  }
}
