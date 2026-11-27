use super::*;

pub(super) fn user_name() -> String {
  std::env::var("USERNAME")
    .ok()
    .filter(|u| !u.is_empty())
    .unwrap_or_else(|| "nexora".to_string())
}

impl Win {
  pub(super) fn go(&mut self, t: u8, push: bool) {
    if push && t != self.tab {
      self.history.truncate(self.hpos + 1);
      self.history.push(t);
      self.hpos = self.history.len() - 1;
    }
    self.tab = t;
    self.scroll = 0.0;
    self.render();
  }
  pub(super) fn page_matches(&self, idx: u8, name: &str) -> bool {
    let q = self.search.trim().to_lowercase();
    if q.is_empty() || name.to_lowercase().contains(&q) {
      return true;
    }
    self.page(idx).iter().any(|(h, rows)| {
      h.is_some_and(|h| h.to_lowercase().contains(&q)) || rows.iter().any(|r| row_label(r).to_lowercase().contains(&q))
    })
  }
  pub(super) fn region(&mut self, x: f32, y: f32, w: f32, h: f32, a: Action) {
    self.regions.push(Region {
      x,
      y,
      w,
      h,
      a,
      scroll: true,
    });
  }
  pub(super) fn hover_side(&mut self, hit: Option<u8>) {
    if hit == self.side_hover {
      return;
    }
    for (page, on) in [(self.side_hover, false), (hit, true)] {
      let Some((_, r, ic)) = self.side.iter().find(|e| Some(e.0) == page) else {
        continue;
      };
      let to = if on { crate::motion::HOVER_ROW } else { 1.0 };
      let _ = crate::motion::hover_pop(
        &self.comp,
        ic,
        Vector3 {
          X: r.0 + r.2 / 2.0,
          Y: r.1 + r.3 / 2.0,
          Z: 0.0,
        },
        to,
      );
    }
    self.side_hover = hit;
  }
  pub(super) fn region_fixed(&mut self, x: f32, y: f32, w: f32, h: f32, a: Action) {
    self.regions.push(Region {
      x,
      y,
      w,
      h,
      a,
      scroll: false,
    });
  }
  pub(super) fn spec(id: u8) -> (f32, f32, &'static str) {
    match id {
      4 => (32.0, 72.0, " dp"),
      5 => (0.0, 24.0, " dp"),
      6 => (100.0, 250.0, "%"),
      7 => (0.0, 60.0, " dp"),
      8 => (70.0, 260.0, " dp"),
      9 => (0.0, 60.0, " dp"),
      10 => (2.0, 10.0, ""),
      11 => (50.0, 200.0, "%"),
      12 => (75.0, 150.0, "%"),
      13 => (0.0, 100.0, "%"),
      14 => (0.0, 100.0, "%"),
      15 => (0.0, 100.0, "%"),
      16 => (0.0, 100.0, "%"),
      17 => (24.0, 100.0, " dp"),
      18 => (0.0, 60.0, " dp"),
      19 => (0.0, 60.0, " dp"),
      20 => (1.0, 100.0, " dp"),
      21 => (1.0, 30.0, " dp"),
      22 => (0.0, 40.0, " dp"),
      23 => (0.0, 1500.0, " ms"),
      24 => (0.0, 100.0, " dp"),
      25 => (0.0, 1500.0, " ms"),
      26 => (1.0, 15.0, " s"),
      27 => (0.0, 30.0, " s"),
      // Appearance sliders (APP-02..APP-33)
      28 | 40 | 44 | 47 | 50 => (0.0, 50.0, ""),
      29 | 31 | 35 | 36 | 38 | 39 | 41 | 42 | 45 | 46 | 48 | 49 => (0.0, 100.0, "%"),
      30 | 33 => (0.0, 30.0, ""),
      32 => (0.0, 20.0, " dp"),
      34 => (0.0, 40.0, ""),
      37 => (0.0, 50.0, ""),
      43 => (24.0, 96.0, " dp"),
      _ => (0.0, 100.0, "%"),
    }
  }
  pub(super) fn commit_edit(&mut self) {
    let Some(f) = self.edit.take() else { return };

    // Handle list text fields (8-11)
    if (8..=11).contains(&f) {
      let kind = match f {
        8 => ListKind::ExcludedPrograms,
        9 => ListKind::ProgramBlacklist,
        10 => ListKind::MinimizeBlacklist,
        11 => ListKind::DeletedStartMenuPrograms,
        _ => return,
      };
      prefs::update_lists(|l| l.set_from_text(kind, &self.edit_buf));
      self.render();
      return;
    }

    let t = nexora_core::settings::ShortText::new(&self.edit_buf);
    prefs::update(|s| {
      if f == 0 {
        s.greet_name = t;
      } else {
        s.greet_text = t;
      }
    });
    self.render();
  }
  pub(super) fn row_height(r: &RowDef) -> f32 {
    match r {
      RowDef::Slider(..) | RowDef::GatedSlider(..) | RowDef::WideSlider(..) => 50.0,
      RowDef::Thumbs(..) => 96.0,
      RowDef::Radios(..) => 34.0,
      RowDef::Swatches(..) => 58.0,
      _ => 44.0,
    }
  }
  pub(super) fn card_in(
    &mut self,
    body: &ContainerVisual,
    y: f32,
    rows: &[RowDef],
    look: &nexora_core::settings::Look,
    vt: f32,
  ) -> f32 {
    let before = self.regions.len();
    let used = self.card(body, y, rows, look);
    for r in self.regions[before..].iter_mut() {
      r.y += vt;
    }
    used
  }
  pub(super) fn hit(&self, x: f32, y: f32) -> Option<Action> {
    let d = self.dpi;
    let (vt, vb) = (VIEW_TOP * d, (H - 10.0) * d);
    self
      .regions
      .iter()
      .rev()
      .find(|r| {
        let yy = if r.scroll {
          if y < vt || y > vb {
            return false;
          }
          y + self.scroll
        } else {
          y
        };
        x >= r.x && x < r.x + r.w && yy >= r.y && yy < r.y + r.h
      })
      .map(|r| r.a)
  }
  pub(super) fn slide(
    &self,
    id: u8,
    x: f32,
    sx: f32,
    sw: f32,
    save: bool,
    scale: Option<nexora_core::settings::NumericSlider>,
  ) {
    let position = slider::value_at(x, sx, sw, false);
    if let Some(scale) = scale {
      let value = scale.position_to_value(position).clamp(0.0, u16::MAX as f32) as u16;
      prefs::update_with(save, |s| match id {
        23 => s.extra.dock_activation_delay = value,
        25 => s.extra.preview_delay = value,
        _ => {}
      });
      return;
    }
    let (min, max, _) = Self::spec(id);
    let v = (min + position * (max - min)).round() as u8;
    prefs::update_with(save, |s| match id {
      0 => s.blur = v,
      1 => s.transparency = v,
      2 => s.dock_round = v,
      3 => s.notch_round = v,
      4 => s.dock_size = v,
      5 => s.dock_gap = v,
      6 => s.dock_zoom = v,
      8 => s.stage_size = v,
      9 => s.stage_gap = v,
      10 => s.stage_count = v,
      11 => s.stage_speed = v,
      12 => s.extra.font_scale = ((v as f32 / 100.0) * 75.0 + 75.0).round() as u16,
      13 => s.extra.screen_brightness = v as u16,
      14 => s.extra.night_light_intensity = v as u16,
      15 => s.extra.dock_icon_light_intensity = v as u16,
      16 => s.extra.dock_icon_background_light = v as u16,
      17 => s.extra.launchpad_icon_size = v as u16,
      18 => s.extra.launchpad_h_spacing = v as u16,
      19 => s.extra.launchpad_v_spacing = v as u16,
      20 => s.extra.dock_act_bar_width = v as u16,
      21 => s.extra.dock_act_bar_height = v as u16,
      22 => s.extra.dock_act_bar_distance = v as u16,
      23 => s.extra.dock_activation_delay = (v as u16) * 6,
      24 => s.extra.preview_size = (v as u16) * 4 + 120,
      25 => s.extra.preview_delay = (v as u16) * 6,
      26 => s.extra.finder_notification_seconds = v as u16,
      27 => s.extra.new_tray_icon_seconds = v as u16,
      28 => s.extra.dock_blur = v as u16,
      29 => s.extra.dock_transparency = v as u16,
      30 => s.extra.reflection_blur = v as u16,
      31 => s.extra.reflection_transparency = v as u16,
      32 => s.extra.reflection_distance = v as u16,
      33 => s.extra.reflection_gradient_distance = v as u16,
      34 => s.extra.label_blur = v as u16,
      35 => s.extra.label_transparency = v as u16,
      36 => s.extra.label_border_transparency = v as u16,
      37 => s.extra.context_blur = v as u16,
      38 => s.extra.context_transparency = v as u16,
      39 => s.extra.context_border = v as u16,
      40 => s.extra.folder_blur = v as u16,
      41 => s.extra.folder_transparency = v as u16,
      42 => s.extra.folder_border = v as u16,
      43 => s.extra.folder_icon_size = v as u16,
      44 => s.extra.menu_blur = v as u16,
      45 => s.extra.menu_transparency = v as u16,
      46 => s.extra.menu_border = v as u16,
      47 => s.extra.finder_blur = v as u16,
      48 => s.extra.finder_transparency = v as u16,
      49 => s.extra.wallpaper_tint = v as u16,
      50 => s.extra.launchpad_blur = v as u16,
      _ => s.dock_margin = v,
    });
  }
}

pub(super) fn xy(lp: LPARAM) -> (f32, f32) {
  ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32)
}

pub(super) fn with_win<R>(f: impl FnOnce(&mut Win) -> R) -> Option<R> {
  WIN.with(|s| s.try_borrow_mut().ok().and_then(|mut b| b.as_mut().map(|w| f(w))))
}
