use super::*;

impl Win {
  pub(super) fn card(
    &mut self,
    c: &ContainerVisual,
    y0: f32,
    rows: &[RowDef],
    look: &nexora_core::settings::Look,
  ) -> f32 {
    let d = self.dpi;
    let (x0, cw) = (CONTENT_X * d, CONTENT_W * d);
    let total: f32 = rows.iter().map(|r| Self::row_height(r) * d).sum();
    self.rect(
      c,
      x0,
      y0,
      cw,
      total,
      12.0 * d,
      col(look.fg, if look.dark { 22 } else { 16 }),
    );
    let mut y = y0;
    let accent = prefs::accent();
    for (i, r) in rows.iter().enumerate() {
      let rh = Self::row_height(r) * d;
      if i > 0 && !matches!(r, RowDef::Radios(..)) {
        self.rect(c, x0 + 16.0 * d, y, cw - 32.0 * d, 1.0, 0.0, col(look.fg, 28));
      }
      let (lx, rx) = (x0 + 16.0 * d, x0 + cw - 16.0 * d);
      match r {
        RowDef::Toggle(label, on, act) => {
          self.text(c, label, lx, y + (rh - 17.0 * d) / 2.0, 13.5, 400, look.fg, 0.95, 0);
          let (tw, th) = (46.0 * d, 22.0 * d);
          let (tx, ty) = (rx - tw, y + (rh - th) / 2.0);
          self.rect(
            c,
            tx,
            ty,
            tw,
            th,
            th / 2.0,
            if *on { col(accent, 255) } else { col(look.fg, 60) },
          );
          let knob = th - 4.0 * d;
          let pressed = self.press.is_some_and(|(px, py)| {
            px >= x0 && px < x0 + cw && py + self.scroll >= y && py + self.scroll < y + rh && py > VIEW_TOP * d
          });
          let kw = if pressed { knob * 1.8 } else { knob * 1.55 };
          let kx = if *on { tx + tw - kw - 2.0 * d } else { tx + 2.0 * d };
          if pressed {
            self.glass_knob(c, kx, ty + 2.0 * d, kw, knob);
          } else {
            self.rect(
              c,
              kx,
              ty + 2.0 * d,
              kw,
              knob,
              knob / 2.0,
              Color {
                A: 255,
                R: 255,
                G: 255,
                B: 255,
              },
            );
          }
          self.region(x0, y, cw, rh, *act);
        }
        RowDef::Seg(label, opts, sel) => {
          self.text(c, label, lx, y + (rh - 17.0 * d) / 2.0, 13.5, 400, look.fg, 0.95, 0);
          let texts: Vec<f32> = opts
            .iter()
            .map(|(t, _)| {
              measure(
                t,
                &Style {
                  px: (12.5 * d).round() as i32,
                  weight: 500,
                },
              )
              .0 as f32
            })
            .collect();
          let avail = (rx - lx)
            - measure(
              label,
              &Style {
                px: (13.5 * d).round() as i32,
                weight: 400,
              },
            )
            .0 as f32
            - 28.0 * d;
          let need = |pad: f32, min: f32| texts.iter().map(|w| (w + pad).max(min)).sum::<f32>() + 6.0 * d;
          let (pad, min) = if need(32.0 * d, 66.0 * d) <= avail {
            (32.0 * d, 66.0 * d)
          } else {
            (18.0 * d, 48.0 * d)
          };
          let widths: Vec<f32> = texts.iter().map(|w| (w + pad).max(min)).collect();
          let total_w: f32 = widths.iter().sum::<f32>() + 6.0 * d;
          let (bx, bh) = (rx - total_w, 26.0 * d);
          let by = y + (rh - bh) / 2.0;
          self.rect(c, bx, by, total_w, bh, bh / 2.0, col(look.fg, 30));
          let inverse = if look.dark { [10u8, 10, 12] } else { [255u8, 255, 255] };
          let mut sx = bx + 3.0 * d;
          for (i, (t, act)) in opts.iter().enumerate() {
            let on = i == *sel;
            if on {
              self.rect(
                c,
                sx,
                by + 3.0 * d,
                widths[i],
                bh - 6.0 * d,
                (bh - 6.0 * d) / 2.0,
                col(look.fg, 235),
              );
            }
            let rgb = if on { inverse } else { look.fg };
            self.text(
              c,
              t,
              sx + widths[i] / 2.0,
              by + (bh - 16.0 * d) / 2.0 - 1.0,
              12.5,
              if on { 600 } else { 500 },
              rgb,
              if on { 1.0 } else { 0.85 },
              1,
            );
            self.region(sx, y, widths[i], rh, *act);
            sx += widths[i];
          }
        }
        RowDef::Slider(..) | RowDef::GatedSlider(..) | RowDef::WideSlider(..) => {
          let (label, value, id, min, max, unit, enabled, scale) = match r {
            RowDef::Slider(label, value, id) => {
              let (min, max, unit) = Self::spec(*id);
              (*label, *value as f32, *id, min, max, unit, true, None)
            }
            RowDef::GatedSlider(label, value, id, enabled) => {
              let (min, max, unit) = Self::spec(*id);
              (*label, *value as f32, *id, min, max, unit, *enabled, None)
            }
            RowDef::WideSlider(label, value, id, scale, unit) => {
              (*label, *value, *id, scale.min(), scale.max(), *unit, true, Some(*scale))
            }
            _ => unreachable!(),
          };
          let frac = scale.map_or_else(
            || ((value - min) / (max - min)).clamp(0.0, 1.0),
            |scale| scale.value_to_position(value),
          );
          self.text(
            c,
            label,
            lx,
            y + (rh - 17.0 * d) / 2.0,
            13.5,
            400,
            look.fg,
            if enabled { 0.95 } else { 0.4 },
            0,
          );
          let display_value = if value.fract() == 0.0 {
            format!("{value:.0}{unit}")
          } else {
            format!("{value:.2}{unit}")
          };
          self.text(
            c,
            &display_value,
            rx,
            y + (rh - 17.0 * d) / 2.0,
            12.5,
            500,
            look.fg,
            if enabled { 0.6 } else { 0.35 },
            2,
          );
          let (tw, th) = (260.0 * d, 5.0 * d);
          let tx = rx - 56.0 * d - tw;
          let ty = y + rh / 2.0 - th / 2.0;
          self.rect(c, tx, ty, tw, th, th / 2.0, col(look.fg, if enabled { 50 } else { 25 }));
          self.rect(
            c,
            tx,
            ty,
            (tw * frac).max(th),
            th,
            th / 2.0,
            col(accent, if enabled { 255 } else { 70 }),
          );
          let knob = 18.0 * d;
          if enabled && self.dragging.map(|(i, _, _)| i) == Some(id) {
            let (kw, kh) = (knob * 1.9, knob * 1.25);
            let cx = (tx + tw * frac).clamp(tx + kw / 2.0 - knob / 4.0, tx + tw - kw / 2.0 + knob / 4.0);
            self.glass_knob(c, cx - kw / 2.0, ty + th / 2.0 - kh / 2.0, kw, kh);
          } else {
            let kx = (tx + tw * frac - knob / 2.0).clamp(tx - knob / 4.0, tx + tw - knob * 0.75);
            self.rect(
              c,
              kx - 1.0 * d,
              ty + th / 2.0 - knob / 2.0,
              knob + 2.0 * d,
              knob + 2.0 * d,
              knob / 2.0 + d,
              Color {
                A: if enabled { 60 } else { 24 },
                R: 0,
                G: 0,
                B: 0,
              },
            );
            self.rect(
              c,
              kx,
              ty + th / 2.0 - knob / 2.0 - 1.0 * d,
              knob,
              knob,
              knob / 2.0,
              Color {
                A: if enabled { 255 } else { 90 },
                R: 255,
                G: 255,
                B: 255,
              },
            );
          }
          if enabled {
            self.region(
              tx - 8.0 * d,
              y + 4.0 * d,
              tw + 16.0 * d,
              rh - 8.0 * d,
              Action::Slider(id, tx, tw, scale),
            );
          }
        }
        RowDef::Swatches(label, sel) => {
          self.text(c, label, lx, y + 13.0 * d, 13.5, 400, look.fg, 0.95, 0);
          let (dia, gap) = (19.0 * d, 10.0 * d);
          let n = nexora_core::settings::ACCENTS.len() + 1;
          let total_w = n as f32 * (dia + gap) - gap;
          let mut sx = rx - total_w;
          let cy = y + 12.0 * d;
          for (i, &name) in ACCENT_NAMES.iter().enumerate().take(n) {
            if i == 0 {
              let key = format!("multi|{}", dia.round());
              let px = dia.round() as u32;
              if let Some(s) = self.thumb_surface(key, px, px, || nexora_core::preview::multicolor(px)) {
                self.sprite(c, &s, sx, cy, dia, dia);
              }
            } else {
              self.rect(
                c,
                sx,
                cy,
                dia,
                dia,
                dia / 2.0,
                col(nexora_core::settings::ACCENTS[i - 1], 255),
              );
            }
            if i == *sel {
              self.rect(
                c,
                sx + dia * 0.32,
                cy + dia * 0.32,
                dia * 0.36,
                dia * 0.36,
                dia * 0.18,
                Color {
                  A: 255,
                  R: 255,
                  G: 255,
                  B: 255,
                },
              );
              self.text(
                c,
                prefs::tr(name),
                sx + dia / 2.0,
                cy + dia + 5.0 * d,
                10.5,
                500,
                look.fg,
                0.7,
                1,
              );
            }
            let v = if i == 0 {
              nexora_core::settings::ACCENT_MULTI
            } else {
              (i - 1) as u8
            };
            self.region(sx - 2.0 * d, y, dia + 4.0 * d, rh, Action::Accent(v));
            sx += dia + gap;
          }
        }
        RowDef::Thumbs(label, opts, sel) => {
          self.text(c, label, lx, y + 14.0 * d, 13.5, 400, look.fg, 0.95, 0);
          let icon_kind = matches!(opts.first().map(|o| o.1), Some(Thumb::Icon(_)));
          let (tw, th) = if icon_kind {
            (48.0 * d, 48.0 * d)
          } else {
            (70.0 * d, 46.0 * d)
          };
          let gap = 16.0 * d;
          let total = opts.len() as f32 * tw + (opts.len() as f32 - 1.0) * gap;
          let mut x = rx - total;
          let ty = y + 12.0 * d;
          let dark_icons = prefs::icon_dark();
          let s0 = prefs::get();
          for (i, (name, kind, act)) in opts.iter().enumerate() {
            let on = i == *sel;
            let r = if icon_kind { tw * 0.22 } else { th * 0.14 };
            if on {
              self.rect(
                c,
                x - 3.0 * d,
                ty - 3.0 * d,
                tw + 6.0 * d,
                th + 6.0 * d,
                r + 3.0 * d,
                col(accent, 255),
              );
              self.rect(
                c,
                x - 1.0 * d,
                ty - 1.0 * d,
                tw + 2.0 * d,
                th + 2.0 * d,
                r + 1.0 * d,
                col(if look.dark { [30, 30, 34] } else { [246, 246, 248] }, 255),
              );
            }
            let (pw, ph) = (tw.round() as u32, th.round() as u32);
            let surf = match *kind {
              Thumb::Appear(k) => self.thumb_surface(format!("appear|{k}|{pw}x{ph}"), pw, ph, || {
                nexora_core::preview::appearance(k, pw, ph)
              }),
              Thumb::Icon(st) => {
                let tint = s0.icon_tint;
                self.thumb_surface(format!("icon|{st:?}|{tint}|{dark_icons}|{pw}"), pw, pw, || {
                  nexora_core::preview::icon_tile(st, tint, dark_icons, pw)
                })
              }
            };
            if let Some(sf) = surf {
              self.sprite(c, &sf, x, ty, tw, th);
            }
            self.text(
              c,
              name,
              x + tw / 2.0,
              ty + th + 6.0 * d,
              11.5,
              if on { 600 } else { 400 },
              look.fg,
              if on { 1.0 } else { 0.7 },
              1,
            );
            self.region(x - 4.0 * d, y, tw + 8.0 * d, rh, *act);
            x += tw + gap;
          }
        }
        RowDef::Radios(opts, sel) => {
          let rpx = (12.5 * d).round() as i32;
          let dia = 14.0 * d;
          let widths: Vec<f32> = opts
            .iter()
            .map(|(t, _)| dia + 6.0 * d + measure(t, &Style { px: rpx, weight: 400 }).0 as f32 + 16.0 * d)
            .collect();
          let mut x = rx - widths.iter().sum::<f32>() + 16.0 * d;
          let cy = y + rh / 2.0 - 4.0 * d;
          for (i, (t, act)) in opts.iter().enumerate() {
            let on = i == *sel;
            if on {
              self.rect(c, x, cy - dia / 2.0, dia, dia, dia / 2.0, col(accent, 255));
              self.rect(
                c,
                x + dia * 0.32,
                cy - dia * 0.18,
                dia * 0.36,
                dia * 0.36,
                dia * 0.18,
                Color {
                  A: 255,
                  R: 255,
                  G: 255,
                  B: 255,
                },
              );
            } else {
              self.rect(c, x, cy - dia / 2.0, dia, dia, dia / 2.0, col(look.fg, 70));
              self.rect(
                c,
                x + 1.2 * d,
                cy - dia / 2.0 + 1.2 * d,
                dia - 2.4 * d,
                dia - 2.4 * d,
                dia / 2.0 - 1.2 * d,
                col(if look.dark { [44, 44, 48] } else { [244, 244, 246] }, 255),
              );
            }
            self.text(c, t, x + dia + 6.0 * d, cy - 8.0 * d, 12.5, 400, look.fg, 0.9, 0);
            self.region(x - 4.0 * d, y, widths[i], rh, *act);
            x += widths[i];
          }
        }
        RowDef::Text(label, value, hint, field) => {
          self.text(c, label, lx, y + (rh - 17.0 * d) / 2.0, 13.5, 400, look.fg, 0.95, 0);
          let editing = self.edit == Some(*field);
          let (bw, bh) = (240.0 * d, 26.0 * d);
          let (bx, by) = (rx - bw, y + (rh - bh) / 2.0);
          self.rect(c, bx, by, bw, bh, bh / 2.0, col(look.fg, if editing { 40 } else { 26 }));
          if editing {
            self.rect(c, bx, by, bw, bh, bh / 2.0, col(accent, 40));
          }
          let (shown, op) = if editing {
            (format!("{}|", self.edit_buf), 1.0)
          } else if value.is_empty() {
            (hint.clone(), 0.45)
          } else {
            (value.clone(), 0.95)
          };
          self.text(
            c,
            &shown,
            bx + 12.0 * d,
            by + (bh - 16.0 * d) / 2.0 - 1.0,
            12.5,
            500,
            look.fg,
            op,
            0,
          );
          self.region(bx, y, bw, rh, Action::EditText(*field));
        }
        RowDef::Pick(label, value, act) => {
          self.text(c, label, lx, y + (rh - 17.0 * d) / 2.0, 13.5, 400, look.fg, 0.95, 0);
          let vw = (measure(
            value,
            &Style {
              px: (12.5 * d).round() as i32,
              weight: 500,
            },
          )
          .0 as f32
            + 44.0 * d)
            .max(96.0 * d);
          let (bx, bh) = (rx - vw, 26.0 * d);
          let by = y + (rh - bh) / 2.0;
          self.rect(c, bx, by, vw, bh, bh / 2.0, col(look.fg, 30));
          self.text(
            c,
            value,
            bx + 14.0 * d,
            by + (bh - 16.0 * d) / 2.0 - 1.0,
            12.5,
            500,
            look.fg,
            1.0,
            0,
          );
          self.text(
            c,
            "\u{2303}",
            bx + vw - 14.0 * d,
            by + (bh - 16.0 * d) / 2.0,
            10.0,
            600,
            look.fg,
            0.6,
            1,
          );
          self.region(x0, y, cw, rh, *act);
        }
      }
      y += rh;
    }
    total
  }
}
