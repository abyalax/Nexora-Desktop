use super::*;

impl Cc {
  pub(super) fn draw_module(&mut self, c: &ContainerVisual, p: &Placed, r: Rect, dpi: f32, radius: f32) -> Mod {
    let k = p.kind;
    let on = self.is_on(k);
    let px = |v: f32| (v * dpi) as i32;
    let rad = radius.min(r.2 / 2.0).min(r.3 / 2.0);
    let (gt, grc) = (17.0 * dpi, rad + 2.5 * dpi);
    let (gcx, gcy) = (r.0 + r.2 - rad, r.1 + r.3 - rad);
    let grip_box: Rect = (gcx - gt, gcy - gt, grc + 2.0 * gt, grc + 2.0 * gt);
    let mut m = Mod {
      placed: *p,
      rect: r,
      track: None,
      fill: None,
      thumb: vec![],
      btns: vec![],
      badge: (r.0 - 6.0 * dpi, r.1 - 6.0 * dpi, 24.0 * dpi, 24.0 * dpi),
      grip: grip_box,
      vertical: false,
      cont: None,
    };
    let small = p.w == 1 && p.h == 1;
    match k {
      Kind::Display | Kind::Sound if small => {
        let sym = if k == Kind::Display { Sym::Sun } else { Sym::SpeakerWave };
        self.symbol(c, sym, px(20.0), WHITE, r.0 + r.2 / 2.0, r.1 + r.3 / 2.0, 0.95);
        m.btns.push((r, 10));
      }
      Kind::Display | Kind::Sound if p.w == 1 => {
        let tw = 12.0 * dpi;
        let (top, bot) = (r.1 + 16.0 * dpi, r.1 + r.3 - 40.0 * dpi);
        let tx = r.0 + (r.2 - tw) / 2.0;
        let th = (bot - top).max(20.0 * dpi);
        let _ = self.rrect(
          c,
          (tx, top, tw, th),
          tw / 2.0,
          Color {
            A: 45,
            R: 255,
            G: 255,
            B: 255,
          },
        );
        let frac = if k == Kind::Display {
          brightness::latest().map(|v| v as f32 / 100.0).unwrap_or(0.5)
        } else {
          audio::latest().level
        };
        let (kw, kh) = (KNOB.1 * dpi, KNOB.0 * dpi);
        let ky = (top + th * (1.0 - frac)).clamp(top + kh / 2.0, top + th - kh / 2.0);
        let fill = self.rrect(
          c,
          (tx, ky, tw, (top + th - ky).max(tw)),
          tw / 2.0,
          Color {
            A: 240,
            R: 255,
            G: 255,
            B: 255,
          },
        );
        let sym = if k == Kind::Display { Sym::Sun } else { Sym::SpeakerWave };
        self.symbol(c, sym, px(17.0), WHITE, r.0 + r.2 / 2.0, r.1 + r.3 - 21.0 * dpi, 0.85);
        let mut thumb = Vec::new();
        let sh = 1.0 * dpi;
        let [s0, s1, s2] = DROP_IDLE;
        for (inset, col) in [(-sh, s0), (0.0, s1), (1.5 * dpi, s2)] {
          let (w, h) = (kw - 2.0 * inset, kh - 2.0 * inset);
          if let Some(v) = self.rrect(c, (r.0 + r.2 / 2.0 - w / 2.0, ky - h / 2.0, w, h), w / 2.0, col) {
            thumb.push(v);
          }
        }
        m.thumb = thumb;
        m.track = Some((tx - 14.0 * dpi, top, tw + 28.0 * dpi, th));
        m.fill = fill;
        m.vertical = true;
      }
      Kind::Display | Kind::Sound => {
        let title = prefs::tr(k.title());
        let titled = p.w >= 3 || p.h >= 2;
        if titled {
          self.text(
            c,
            title,
            px(15.5),
            600,
            WHITE,
            r.0 + 20.0 * dpi,
            r.1 + 12.0 * dpi,
            r.2 - 40.0 * dpi,
            0.95,
          );
        }
        let th = 10.0 * dpi;
        let out = if k == Kind::Sound && r.2 > 250.0 * dpi {
          36.0 * dpi
        } else {
          0.0
        };
        let right = if out > 0.0 { 108.0 * dpi } else { 40.0 * dpi };
        let cy = if p.h >= 2 {
          r.1 + r.3 - 36.0 * dpi
        } else if titled {
          r.1 + r.3 * 0.65
        } else {
          r.1 + r.3 / 2.0
        };
        let (tx, ty) = (r.0 + 42.0 * dpi, cy - th / 2.0);
        let tw = r.2 - 42.0 * dpi - right;
        let _ = self.rrect(
          c,
          (tx, ty, tw, th),
          th / 2.0,
          Color {
            A: 45,
            R: 255,
            G: 255,
            B: 255,
          },
        );
        let frac = if k == Kind::Display {
          brightness::latest().map(|v| v as f32 / 100.0).unwrap_or(0.5)
        } else {
          audio::latest().level
        };
        let (kw, kh) = (KNOB.0 * dpi, KNOB.1 * dpi);
        let kx = (tx + tw * frac).clamp(tx + kw / 2.0, tx + tw - kw / 2.0);
        let fill = self.rrect(
          c,
          (tx, ty, (kx - tx).max(th), th),
          th / 2.0,
          Color {
            A: 240,
            R: 255,
            G: 255,
            B: 255,
          },
        );
        let (lo, hi, lo_px, hi_px) = if k == Kind::Display {
          (Sym::SunSmall, Sym::Sun, 15, 19)
        } else {
          (Sym::Speaker, Sym::SpeakerWave, 19, 25)
        };
        self.symbol(c, lo, px(lo_px as f32), WHITE, r.0 + 23.0 * dpi, cy, 0.85);
        let hi_x = if out > 0.0 {
          r.0 + r.2 - 86.0 * dpi
        } else {
          r.0 + r.2 - 21.0 * dpi
        };
        self.symbol(c, hi, px(hi_px as f32), WHITE, hi_x, cy, 0.85);
        if out > 0.0 {
          let b: Rect = (r.0 + r.2 - 39.0 * dpi - out / 2.0, cy - out / 2.0, out, out);
          let _ = self.rrect(
            c,
            b,
            out / 2.0,
            Color {
              A: 45,
              R: 255,
              G: 255,
              B: 255,
            },
          );
          self.symbol(c, Sym::Headphones, px(19.0), WHITE, b.0 + out / 2.0, cy, 0.9);
          m.btns.push((b, 10));
        }
        let mut thumb = Vec::new();
        let sh = 1.0 * dpi;
        let [s0, s1, s2] = DROP_IDLE;
        for (inset, col) in [(-sh, s0), (0.0, s1), (1.5 * dpi, s2)] {
          let (w, h) = (kw - 2.0 * inset, kh - 2.0 * inset);
          if let Some(v) = self.rrect(c, (kx - w / 2.0, cy - h / 2.0, w, h), h / 2.0, col) {
            thumb.push(v);
          }
        }
        m.thumb = thumb;
        m.track = Some((tx, ty - 10.0 * dpi, tw, th + 20.0 * dpi));
        m.fill = fill;
      }
      Kind::NowPlaying => {
        let (title, artist, has, playing) = self.track.clone().unwrap_or_default();
        let pad = 14.0 * dpi;
        let play = if playing { Sym::Pause } else { Sym::Play };
        if !has {
          let (cx, cy) = (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
          self.glyph(
            c,
            "\u{E8D6}",
            px(if small { 20.0 } else { 26.0 }),
            WHITE,
            cx,
            if small || p.h == 1 { cy } else { cy - 10.0 * dpi },
            0.6,
          );
          if !small && p.h >= 2 {
            self.text(
              c,
              prefs::tr("Tidak diputar"),
              px(12.0),
              500,
              WHITE,
              r.0 + 16.0 * dpi,
              cy + 14.0 * dpi,
              r.2 - 20.0 * dpi,
              0.7,
            );
          }
        } else if small {
          let (cx, cy) = (r.0 + r.2 / 2.0, r.1 + r.3 / 2.0);
          self.symbol(c, play, px(22.0), WHITE, cx, cy, 1.0);
          m.btns.push((r, 1));
        } else if p.h == 1 {
          let art = r.3 - 2.0 * pad;
          self.art_at(c, r.0 + pad, r.1 + pad, art, dpi);
          let ctrl: Vec<(Sym, u8)> = if p.w >= 3 {
            vec![(Sym::Backward, 0), (play, 1), (Sym::Forward, 2)]
          } else {
            vec![(play, 1)]
          };
          let bw = 38.0 * dpi;
          let cy = r.1 + r.3 / 2.0;
          let x_end = r.0 + r.2 - pad;
          for (j, (sy, id)) in ctrl.iter().enumerate() {
            let cx = x_end - bw * (ctrl.len() - j) as f32 + bw / 2.0;
            self.symbol(c, *sy, px(if *id == 1 { 20.0 } else { 16.0 }), WHITE, cx, cy, 1.0);
            m.btns.push(((cx - bw / 2.0, cy - bw / 2.0, bw, bw), *id));
          }
          let tx = r.0 + pad + art + 10.0 * dpi;
          let tw = x_end - bw * ctrl.len() as f32 - tx - 6.0 * dpi;
          if tw > 30.0 * dpi {
            self.text(c, &title, px(13.0), 600, WHITE, tx, cy - 18.0 * dpi, tw, 1.0);
            self.text(c, &artist, px(12.0), 400, WHITE, tx, cy + 1.0 * dpi, tw, 0.65);
          }
        } else if p.w == 1 {
          let art = r.2 - 2.0 * pad;
          self.art_at(c, r.0 + pad, r.1 + pad, art, dpi);
          let cy = r.1 + r.3 - 30.0 * dpi;
          self.symbol(c, play, px(20.0), WHITE, r.0 + r.2 / 2.0, cy, 1.0);
          m.btns.push(((r.0, cy - 20.0 * dpi, r.2, 40.0 * dpi), 1));
        } else if p.w >= 3 {
          let art = (r.3 - 2.0 * pad).min(r.2 * 0.45);
          self.art_at(c, r.0 + pad, r.1 + pad, art, dpi);
          let tx = r.0 + pad + art + 14.0 * dpi;
          let tw = r.0 + r.2 - pad - tx;
          self.text(c, &title, px(15.0), 600, WHITE, tx, r.1 + pad + 4.0 * dpi, tw, 1.0);
          self.text(c, &artist, px(13.0), 400, WHITE, tx, r.1 + pad + 26.0 * dpi, tw, 0.65);
          let cy = r.1 + r.3 - pad - 22.0 * dpi;
          let syms = [Sym::Backward, play, Sym::Forward];
          for (i, sy) in syms.iter().enumerate() {
            let cx = tx + tw * (0.18 + i as f32 * 0.32);
            self.symbol(c, *sy, px(if i == 1 { 24.0 } else { 19.0 }), WHITE, cx, cy, 1.0);
            m.btns
              .push(((cx - 22.0 * dpi, cy - 22.0 * dpi, 44.0 * dpi, 44.0 * dpi), i as u8));
          }
        } else {
          let art = (r.3 - 120.0 * dpi)
            .min(r.2 - 2.0 * pad)
            .max(40.0 * dpi)
            .min(r.2 * 0.24 * p.h as f32);
          self.art_at(c, r.0 + pad, r.1 + pad, art, dpi);
          self.text(
            c,
            &title,
            px(13.0),
            600,
            WHITE,
            r.0 + 16.0 * dpi,
            r.1 + 18.0 * dpi + art,
            r.2 - 28.0 * dpi,
            1.0,
          );
          self.text(
            c,
            &artist,
            px(12.0),
            400,
            WHITE,
            r.0 + 16.0 * dpi,
            r.1 + 38.0 * dpi + art,
            r.2 - 28.0 * dpi,
            0.65,
          );
          let cy = r.1 + r.3 - 24.0 * dpi;
          let cxs = [r.0 + r.2 * 0.22, r.0 + r.2 * 0.5, r.0 + r.2 * 0.78];
          let syms = [Sym::Backward, play, Sym::Forward];
          for (i, (cx, sy)) in cxs.iter().zip(syms).enumerate() {
            self.symbol(c, sy, px(if i == 1 { 20.0 } else { 17.0 }), WHITE, *cx, cy, 1.0);
            m.btns
              .push(((cx - 20.0 * dpi, cy - 20.0 * dpi, 40.0 * dpi, 40.0 * dpi), i as u8));
          }
        }
      }
      _ if p.h >= 2 => {
        let d = 46.0 * dpi;
        let disc: Rect = (r.0 + 12.0 * dpi, r.1 + 12.0 * dpi, d, d);
        let (bg, fg) = if on {
          (
            Color {
              A: 245,
              R: 255,
              G: 255,
              B: 255,
            },
            prefs::accent(),
          )
        } else {
          (
            Color {
              A: 70,
              R: 0,
              G: 0,
              B: 0,
            },
            WHITE,
          )
        };
        let _ = self.rrect(c, disc, d / 2.0, bg);
        self.glyph(c, glyph_of(k), px(20.0), fg, disc.0 + d / 2.0, disc.1 + d / 2.0, 1.0);
        m.btns.push((disc, BTN_DISC));
        let title = prefs::tr(k.title());
        self.text(
          c,
          title,
          px(14.0),
          600,
          WHITE,
          r.0 + 16.0 * dpi,
          r.1 + r.3 - 48.0 * dpi,
          r.2 - 28.0 * dpi,
          1.0,
        );
        let status = self.status(k);
        if !status.is_empty() {
          self.text(
            c,
            &status,
            px(12.0),
            400,
            WHITE,
            r.0 + 16.0 * dpi,
            r.1 + r.3 - 28.0 * dpi,
            r.2 - 28.0 * dpi,
            0.65,
          );
        }
      }
      _ if p.w == 1 && p.h == 1 => {
        if on {
          let _ = self.rrect(c, r, r.2 / 2.0, prefs::accent_c(245));
        }
        self.glyph(c, glyph_of(k), px(20.0), WHITE, r.0 + r.2 / 2.0, r.1 + r.3 / 2.0, 1.0);
      }
      _ => {
        let d = r.3 * 0.58;
        let off = (r.3 - d) / 2.0;
        let disc: Rect = (r.0 + off, r.1 + off, d, d);
        let (bg, fg) = if on {
          (
            Color {
              A: 245,
              R: 255,
              G: 255,
              B: 255,
            },
            prefs::accent(),
          )
        } else {
          (
            Color {
              A: 70,
              R: 0,
              G: 0,
              B: 0,
            },
            WHITE,
          )
        };
        let _ = self.rrect(c, disc, d / 2.0, bg);
        self.glyph(c, glyph_of(k), px(16.0), fg, disc.0 + d / 2.0, disc.1 + d / 2.0, 1.0);
        m.btns.push((disc, BTN_DISC));
        let tx = disc.0 + d + 11.0 * dpi;
        let title = prefs::tr(k.title());
        let status = self.status(k);
        let ty = r.1 + r.3 / 2.0 - if status.is_empty() { 9.5 } else { 19.0 } * dpi;
        self.text(c, title, px(14.0), 600, WHITE, tx, ty, r.2 - d - off - 22.0 * dpi, 1.0);
        if !status.is_empty() {
          self.text(
            c,
            &status,
            px(12.0),
            400,
            WHITE,
            tx,
            ty + 19.0 * dpi,
            r.2 - d - off - 22.0 * dpi,
            0.65,
          );
        }
      }
    }
    if self.edit {
      let b = m.badge;
      let _ = self.rrect(
        c,
        b,
        b.2 / 2.0,
        Color {
          A: 150,
          R: 70,
          G: 70,
          B: 74,
        },
      );
      let _ = self.rrect(
        c,
        b,
        b.2 / 2.0,
        Color {
          A: 46,
          R: 255,
          G: 255,
          B: 255,
        },
      );
      self.glass_ring(c, b, dpi);
      let _ = self.rrect(
        c,
        (
          b.0 + 6.0 * dpi,
          b.1 + b.3 / 2.0 - 1.0 * dpi,
          b.2 - 12.0 * dpi,
          2.2 * dpi,
        ),
        dpi,
        Color {
          A: 255,
          R: 255,
          G: 255,
          B: 255,
        },
      );
      let (bw, bh) = (grip_box.2.ceil() as u32, grip_box.3.ceil() as u32);
      let data = nexora_core::logo::render_grip(bw, bh, (gt, gt), grc, gt, 14.0, 76.0);
      if let Some(Ok(sf)) = self.gfx.as_ref().map(|g| g.surface_from_bgra(bw, bh, &data)) {
        self.put(c, &sf, grip_box.0, grip_box.1, bw as f32, bh as f32, 1.0);
      }
    }
    m
  }
}
