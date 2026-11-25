use super::*;

impl Mgr {
  pub(super) fn spawn(&mut self, kind: Kind, pos: (i32, i32), pop: bool) -> Option<usize> {
    let d = self.dpi;
    let (w, h) = widgets::size_of(kind, d);
    let m = MARGIN * d;
    let base = GlassParams::dark((w, h), RADIUS * d);
    let glass = if style() == 1 {
      prefs::glass((w, h), RADIUS * d, prefs::Role::Dock)
    } else {
      base
    };
    let mut popup = Popup::new(
      &self.comp,
      w!("NexoraWidget"),
      Some(widget_proc),
      WINDOW_EX_STYLE(0),
      glass,
    )
    .ok()?;
    let _ = popup.set_rect(
      pos.0 - m as i32,
      pos.1 - m as i32,
      (w + 2.0 * m) as i32,
      (h + 2.0 * m) as i32,
    );
    if style() == 1 {
      let c = sys::conditions();
      let _ = popup.panel.set_tier(if !c.transparency || c.battery_saver {
        Tier::Solid
      } else {
        Tier::Full
      });
      let _ = popup.panel.set_rect(m, m, w, h);
      let _ = popup.panel.set_radius(RADIUS * d);
    } else {
      let _ = popup.panel.set_rect(0.0, 0.0, 0.0, 0.0);
    }
    let _ = popup.content.SetOffset(Vector3 { X: m, Y: m, Z: 0.0 });
    unsafe {
      let _ = SetWindowPos(
        popup.hwnd,
        Some(HWND_BOTTOM),
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
      );
    }
    popup.show();
    self.list.push(Widget {
      kind,
      popup,
      pos,
      size: (w, h),
      hands: vec![],
      sig: String::new(),
      hover: false,
      tracking: false,
    });
    let i = self.list.len() - 1;
    self.paint(i);
    if pop {
      let wd = &self.list[i];
      let _ = motion::pop_in(&self.comp, wd.popup.root(), self.center(i), 0.4, 1.06, 0, 380);
    }
    Some(i)
  }

  pub(super) fn center(&self, i: usize) -> Vector3 {
    let w = &self.list[i];
    let m = MARGIN * self.dpi;
    Vector3 {
      X: m + w.size.0 / 2.0,
      Y: m + w.size.1 / 2.0,
      Z: 0.0,
    }
  }

  pub(super) fn sig_for(&self, kind: Kind) -> String {
    let n = time::local_now();
    match kind {
      Kind::Clock => format!("{}", n.day),
      Kind::WorldClock => format!("{}{}", n.day, n.hour),
      Kind::Calendar | Kind::Month => format!("{}-{}-{}", n.year, n.month, n.day),
      Kind::Digital => format!("{}{}{}", n.day, n.hour, n.minute),
      Kind::Battery => format!("{:?}", sys::battery()),
      Kind::System => format!("{}-{}", self.cpu, self.ram),
    }
  }

  pub(super) fn paint(&mut self, i: usize) {
    let kind = self.list[i].kind;
    let (w, h) = self.list[i].size;
    let content = self.list[i].popup.content.clone();
    if let Ok(k) = content.Children() {
      let _ = k.RemoveAll();
    }
    let hands = self.draw(&content, kind, w, h, false);
    if self.edit {
      self.badge(&content);
    }
    let sig = self.sig_for(kind);
    let wd = &mut self.list[i];
    wd.hands = hands;
    wd.sig = sig;
    self.turn_hands(i, false);
  }

  pub(super) fn badge(&self, c: &ContainerVisual) {
    let d = self.dpi;
    let s = 22.0 * d;
    let _ = self.rr(
      c,
      (-7.0 * d, -7.0 * d, s, s),
      s / 2.0,
      Color {
        A: 240,
        R: 72,
        G: 72,
        B: 76,
      },
    );
    let _ = self.rr(
      c,
      (-7.0 * d + 6.0 * d, -7.0 * d + s / 2.0 - 1.0 * d, s - 12.0 * d, 2.0 * d),
      d,
      Color {
        A: 255,
        R: 255,
        G: 255,
        B: 255,
      },
    );
  }

  pub(super) fn badge_hit(&self, i: usize, x: f32, y: f32) -> bool {
    let d = self.dpi;
    let m = MARGIN * d;
    self.edit && hit((m - 9.0 * d, m - 9.0 * d, 26.0 * d, 26.0 * d), x, y) && i < self.list.len()
  }
}
