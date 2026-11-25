use super::*;

impl Mgr {
  pub(super) fn draw(&mut self, p: &ContainerVisual, kind: Kind, w: f32, h: f32, preview: bool) -> Vec<Hand> {
    let d = self.dpi;
    let radius = RADIUS * d;
    let (card, fg) = palette(kind);
    match card {
      Some(c) => {
        self.shadow(p, w, h, radius, if preview { 0.22 } else { 0.34 });
        let _ = self.rr(p, (0.0, 0.0, w, h), radius, c);
      }
      None if preview => {
        let _ = self.rr(
          p,
          (0.0, 0.0, w, h),
          radius,
          Color {
            A: 70,
            R: 255,
            G: 255,
            B: 255,
          },
        );
      }
      None => {}
    }
    let mut hands = Vec::new();
    let n = time::local_now();
    match kind {
      Kind::Clock => {
        let r = (w.min(h) - 24.0 * d) / 2.0;
        self.face(p, w / 2.0, h / 2.0, r, None, true, &mut hands);
      }
      Kind::WorldClock => {
        let pad = 12.0 * d;
        let r = ((w - 2.0 * pad - 3.0 * 10.0 * d) / 8.0).min((h - 54.0 * d) / 2.0);
        let slot = (w - 2.0 * pad) / 4.0;
        let utc = utc_minutes();
        let local = sys::utc_offset_min();
        for (k, city) in widgets::CITIES.iter().enumerate() {
          let cx = pad + slot * (k as f32 + 0.5);
          let cy = 14.0 * d + r;
          self.face(p, cx, cy, r, Some(city.offset_min), false, &mut hands);
          let (_, _, day, diff) = widgets::city_time(utc, local, city.offset_min);
          let when = match day {
            0 => prefs::tr("Hari ini"),
            1 => prefs::tr("Besok"),
            _ => prefs::tr("Kemarin"),
          };
          self.text(p, city.name, 11.5 * d, 600, fg, cx, cy + r + 7.0 * d, 1, 1.0);
          let rel = if diff == 0 {
            when.to_string()
          } else {
            format!("{when} {diff:+}{}", prefs::tr("JAM"))
          };
          self.text(p, &rel, 9.5 * d, 500, fg, cx, cy + r + 23.0 * d, 1, 0.6);
        }
      }
      Kind::Calendar => {
        let wd = calendar::weekday(n.year, n.month, n.day);
        self.text(
          p,
          &weekday_label(wd).to_uppercase(),
          12.5 * d,
          700,
          RED,
          16.0 * d,
          14.0 * d,
          0,
          1.0,
        );
        self.text(p, &n.day.to_string(), 52.0 * d, 300, fg, 13.0 * d, 30.0 * d, 0, 1.0);
        self.text(
          p,
          &format!("{} {}", month_label(n.month), n.year),
          12.5 * d,
          500,
          fg,
          16.0 * d,
          h - 32.0 * d,
          0,
          0.6,
        );
      }
      Kind::Month => {
        let g = calendar::grid(n.year, n.month, !en());
        self.text(
          p,
          &month_label(n.month).to_uppercase(),
          11.0 * d,
          700,
          RED,
          14.0 * d,
          11.0 * d,
          0,
          1.0,
        );
        let labels = calendar::weekday_labels(!en());
        let (gx, gy) = (10.0 * d, 30.0 * d);
        let cw = (w - 2.0 * gx) / 7.0;
        let rows = g.rows.max(5) as f32;
        let ch = (h - gy - 8.0 * d) / (rows + 1.0);
        for (k, l) in labels.iter().enumerate() {
          let s: String = l.chars().take(1).collect();
          self.text(p, &s, 9.0 * d, 600, fg, gx + cw * (k as f32 + 0.5), gy, 1, 0.5);
        }
        for day in 1..=g.days {
          let idx = g.first_col + day - 1;
          let (cx, cy) = (gx + cw * ((idx % 7) as f32 + 0.5), gy + ch * ((idx / 7) as f32 + 1.0));
          let today = day == n.day;
          if today {
            let s = ch.min(cw) * 0.92;
            let _ = self.rr(p, (cx - s / 2.0, cy - 1.5 * d, s, s), s / 2.0, col(RED, 255));
          }
          self.text(
            p,
            &day.to_string(),
            9.5 * d,
            if today { 700 } else { 500 },
            if today { [255, 255, 255] } else { fg },
            cx,
            cy,
            1,
            1.0,
          );
        }
      }
      Kind::Digital => {
        let wd = calendar::weekday(n.year, n.month, n.day);
        self.text(
          p,
          &hhmm(n.hour, n.minute),
          46.0 * d,
          600,
          fg,
          w / 2.0,
          h / 2.0 - 40.0 * d,
          1,
          1.0,
        );
        self.text(
          p,
          &weekday_label(wd),
          13.0 * d,
          600,
          ORANGE,
          w / 2.0,
          h / 2.0 + 18.0 * d,
          1,
          1.0,
        );
        self.text(
          p,
          &format!("{} {}", n.day, month_label(n.month)),
          12.5 * d,
          500,
          fg,
          w / 2.0,
          h / 2.0 + 36.0 * d,
          1,
          0.6,
        );
      }
      Kind::Battery => match sys::battery() {
        Some((pct, charging)) => {
          let n_ring = (58.0 * d).round() as u32;
          let ring_c = if pct <= 20 && !charging { RED } else { GREEN };
          let track = if fg[0] > 128 {
            [255, 255, 255, 46]
          } else {
            [0, 0, 0, 30]
          };
          let data = widgets::ring(n_ring, 5.5 * d, pct as f32 / 100.0, ring_c, track);
          if let Some(Ok(s)) = self.gfx.as_ref().map(|g| g.surface_from_bgra(n_ring, n_ring, &data)) {
            self.put(p, &s, 16.0 * d, 16.0 * d, n_ring as f32, n_ring as f32, 1.0);
          }
          let c = 16.0 * d + n_ring as f32 / 2.0;
          self.glyph(
            p,
            if charging { "\u{E83E}" } else { "\u{E7F8}" },
            17.0 * d,
            fg,
            c,
            c,
            0.9,
          );
          self.text(p, &format!("{pct}%"), 34.0 * d, 600, fg, 16.0 * d, h - 52.0 * d, 0, 1.0);
        }
        None => {
          self.glyph(p, "\u{E83F}", 26.0 * d, fg, w / 2.0, h / 2.0 - 14.0 * d, 0.8);
          self.text(
            p,
            prefs::tr("Tidak ada baterai"),
            12.5 * d,
            500,
            fg,
            w / 2.0,
            h / 2.0 + 14.0 * d,
            1,
            0.65,
          );
        }
      },
      Kind::System => {
        self.text(p, prefs::tr("Sistem"), 13.0 * d, 700, fg, 16.0 * d, 14.0 * d, 0, 1.0);
        let rows = [("CPU", self.cpu, BLUE), ("RAM", self.ram, GREEN)];
        for (k, (label, v, c)) in rows.iter().enumerate() {
          let y = 46.0 * d + k as f32 * 52.0 * d;
          self.text(p, label, 12.0 * d, 600, fg, 16.0 * d, y, 0, 0.7);
          self.text(
            p,
            &format!("{v}%"),
            15.0 * d,
            700,
            fg,
            w - 16.0 * d,
            y - 2.0 * d,
            2,
            1.0,
          );
          let bar = (16.0 * d, y + 22.0 * d, w - 32.0 * d, 8.0 * d);
          let track = if fg[0] > 128 {
            Color {
              A: 46,
              R: 255,
              G: 255,
              B: 255,
            }
          } else {
            Color {
              A: 26,
              R: 0,
              G: 0,
              B: 0,
            }
          };
          let _ = self.rr(p, bar, 4.0 * d, track);
          let fw = (bar.2 * *v as f32 / 100.0).max(8.0 * d);
          let _ = self.rr(p, (bar.0, bar.1, fw, bar.3), 4.0 * d, col(*c, 255));
        }
      }
    }
    hands
  }

  pub(super) fn face(
    &mut self,
    p: &ContainerVisual,
    cx: f32,
    cy: f32,
    r: f32,
    offset: Option<i32>,
    numbers: bool,
    hands: &mut Vec<Hand>,
  ) {
    let ink = [24, 24, 26];
    let _ = self.rr(
      p,
      (cx - r, cy - r, 2.0 * r, 2.0 * r),
      r,
      Color {
        A: 255,
        R: 252,
        G: 252,
        B: 253,
      },
    );
    if numbers {
      for i in 1..=12u32 {
        let a = (i as f32 * 30.0).to_radians();
        let rr = r * 0.78;
        let px = (r * 0.2).max(8.0);
        self.text(
          p,
          &i.to_string(),
          px,
          500,
          ink,
          cx + a.sin() * rr,
          cy - a.cos() * rr - px * 0.62,
          1,
          1.0,
        );
      }
    } else {
      for i in 0..12u32 {
        let a = (i as f32 * 30.0).to_radians();
        let (len, th) = if i % 3 == 0 {
          (r * 0.16, (r * 0.05).max(1.5))
        } else {
          (r * 0.09, (r * 0.03).max(1.0))
        };
        if let Some(t) = self.rr(p, (cx - th / 2.0, cy - r + r * 0.08, th, len), th / 2.0, col(ink, 200)) {
          let _ = t.SetCenterPoint(Vector3 {
            X: th / 2.0,
            Y: r - r * 0.08,
            Z: 0.0,
          });
          let _ = t.SetRotationAngleInDegrees(a.to_degrees());
        }
      }
    }
    let specs: [(u8, f32, f32, f32, [u8; 3]); 3] = [
      (0, r * 0.07, r * 0.5, 0.0, ink),
      (1, r * 0.05, r * 0.78, 0.0, ink),
      (2, r * 0.022, r * 0.86, r * 0.18, ORANGE),
    ];
    for (which, th, len, tail, c) in specs {
      let th = th.max(1.2);
      if let Some(v) = self.rr(p, (cx - th / 2.0, cy - len, th, len + tail), th / 2.0, col(c, 255)) {
        let _ = v.SetCenterPoint(Vector3 {
          X: th / 2.0,
          Y: len,
          Z: 0.0,
        });
        hands.push(Hand { v, which, offset });
      }
    }
    let dot = (r * 0.12).max(4.0);
    let _ = self.rr(
      p,
      (cx - dot / 2.0, cy - dot / 2.0, dot, dot),
      dot / 2.0,
      col(ORANGE, 255),
    );
  }

  pub(super) fn turn_hands(&mut self, i: usize, animate: bool) {
    let n = time::local_now();
    let utc = utc_minutes();
    let local = sys::utc_offset_min();
    let comp = self.comp.clone();
    let Some(wd) = self.list.get(i) else { return };
    for h in &wd.hands {
      let (hh, mm) = match h.offset {
        None => (n.hour, n.minute),
        Some(o) => {
          let (a, b, _, _) = widgets::city_time(utc, local, o);
          (a, b)
        }
      };
      let (ha, ma, sa) = widgets::hands(hh, mm, n.second);
      let to = match h.which {
        0 => ha,
        1 => ma,
        _ => sa,
      };
      if h.which == 2 && animate {
        let _ = sweep(&comp, &h.v, to, to + 6.0);
      } else {
        let _ = h.v.SetRotationAngleInDegrees(to);
      }
    }
  }
}
