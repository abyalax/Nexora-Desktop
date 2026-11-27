//! Finder bar item layout, application label and clock text.

use super::*;

/// Measures bar text with the OS font stack for the core item builder.
pub(super) struct BarMetrics<'a>(pub(super) &'a Bar);

impl TextMetrics for BarMetrics<'_> {
  fn text(&self, text: &str, emphasized: bool) -> f32 {
    self.0.text_w(text, if emphasized { APP_W } else { BAR_W })
  }

  fn small(&self, text: &str) -> f32 {
    self.0.small_w(text, 12.0, BAR_W)
  }
}

impl Bar {
  pub(super) fn refresh_app(&mut self) -> bool {
    self.refresh_app_for(focus::foreground())
  }

  pub(super) fn refresh_app_for(&mut self, fg: isize) -> bool {
    if fg == 0 || fg == self.popup.hwnd.0 as isize {
      return false;
    }
    let mut changed = false;
    self.check_fullscreen();
    let native = nexora_platform::menus::read(fg).unwrap_or_default();
    let titles = |v: &Vec<nexora_platform::menus::Top>| v.iter().map(|t| t.title.clone()).collect::<Vec<_>>();
    if titles(&native) != titles(&self.native) {
      changed = true;
    }
    self.native = native;
    self.native_hwnd = fg;
    let wins = enumerate::list(std::process::id());
    let name = wins
      .iter()
      .find(|w| w.hwnd == fg)
      .map(|w| {
        if !w.name.is_empty() {
          w.name.clone()
        } else {
          nexora_core::dock::items::name_from_path(&w.path)
        }
      })
      .filter(|n| !n.is_empty());
    let Some(name) = name else { return changed };
    if name == self.app {
      return changed;
    }
    self.app = name;
    true
  }

  pub(super) fn text_w(&self, s: &str, weight: i32) -> f32 {
    let px = (13.0 * self.dpi).round() as i32;
    measure(s, &Style { px, weight }).0 as f32 + 2.0
  }

  pub(super) fn relayout(&mut self) {
    let d = self.dpi;
    let static_labels = [
      mt("Berkas"),
      mt("Edit"),
      mt("Menu:Tampilan"),
      mt("Menu:Jendela"),
      mt("Bantuan"),
    ];
    let native_titles: Vec<String> = self.native.iter().map(|t| t.title.clone()).collect();
    let app_label = self.app_label();
    let clock = self.clock_text();
    let battery = sys::battery().map(|(pct, _)| pct);
    let inputs = BarInputs {
      dpi: d,
      bar_height: self.h as f32,
      app_label: &app_label,
      menus: if native_titles.is_empty() {
        Menus::Static(&static_labels)
      } else {
        Menus::Native(&native_titles)
      },
      clock: &clock,
      show_stage: prefs::get().stage,
      has_wifi: self.wifi.is_some(),
      battery,
      battery_beside: prefs::get().battery_pct == 2,
    };
    let settings = prefs::get();
    let (left, right) = items::build_with(
      &inputs,
      &Show::from_settings(&settings),
      &items::Available::default(),
      &BarMetrics(self),
    );
    let gap = services::notch().map(|(l, r, _)| Gap {
      left: l as f32,
      right: r as f32,
    });
    let (l, r) = lay::layout(self.screen.0 as f32, ITEM_MARGIN * d, &left, &right, gap);
    self.left = l;
    self.right = r;
    let want_h = services::notch()
      .map(|(_, _, h)| h)
      .unwrap_or((26.0 * d).round() as i32);
    if want_h != self.h {
      self.h = want_h;
      let _ = self.popup.set_rect(0, 0, self.screen.0, self.h);
      let _ = self.popup.panel.set_radius(0.0);
      self.apply_bg();
      shell::appbar::reserve_top(self.popup.hwnd, self.screen.0, self.h);
    }
  }

  pub(super) fn app_label(&self) -> String {
    if self.app.is_empty() {
      "nexora".to_string()
    } else {
      self.app.clone()
    }
  }

  pub(super) fn clock_text(&self) -> String {
    let n = time::local_now();
    let when = nexora_core::menubar::clock::When {
      year: n.year,
      month: n.month,
      day: n.day,
      hour: n.hour,
      minute: n.minute,
    };
    let settings = prefs::get();
    nexora_core::finder::clock::format_clock(
      when,
      settings.lang,
      nexora_core::finder::clock::ClockStyle {
        show_time: settings.extra.finder_show_time,
        show_date: settings.extra.finder_show_date,
        time: nexora_core::finder::clock::TimeFormat::from_legacy(settings.clock_24h),
        date: nexora_core::finder::clock::DateFormat::from_index(settings.extra.finder_date_format),
      },
    )
  }
}
