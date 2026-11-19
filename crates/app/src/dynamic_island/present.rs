//! Look transitions, sizing, region and the render pass.

use super::*;

impl Notch {
  pub(super) fn springs(old: Look, new: Look) -> (f32, u32) {
    let big = |v: View| matches!(v, View::Open(_) | View::Drop);
    if big(new.view) && !big(old.view) {
      (motion::OPEN_RATIO, motion::OPEN_PERIOD_MS)
    } else if big(old.view) && !big(new.view) {
      (motion::CLOSE_RATIO, motion::CLOSE_PERIOD_MS)
    } else if old.view == new.view {
      (motion::PEEK_RATIO, motion::PEEK_PERIOD_MS)
    } else {
      (motion::SWAP_RATIO, motion::SWAP_PERIOD_MS)
    }
  }

  pub(super) fn transition(&mut self, new: Look) {
    let old = self.look;
    let first = self.first;
    self.first = false;
    self.look = new;
    let view_changed = old.view != new.view || first;
    if !matches!(new.view, View::Open(_)) {
      self.cal_sel = 0;
    }
    let springy = (!first).then(|| Self::springs(old, new));
    if view_changed && !first {
      let closing = matches!(old.view, View::Open(_) | View::Drop) && !matches!(new.view, View::Open(_) | View::Drop);
      self.retire_content(if closing {
        motion::CLOSE_FADE_MS
      } else {
        motion::SWAP_FADE_MS
      });
    }
    self.apply_size(springy);
    self.render();
    self.sync_glance();
    if view_changed && !first {
      self.intro(matches!(new.view, View::Open(_) | View::Drop) && !matches!(old.view, View::Open(_) | View::Drop));
    }
  }

  pub(super) fn apply_size(&mut self, springy: Option<(f32, u32)>) {
    let (px0, pw, ph) = (self.x0(), self.size.w, self.size.h);
    let mut z = state::size(self.look, self.dpi);
    z.radius *= crate::prefs::round(crate::prefs::Role::Notch);
    z.w = z.w.round();
    z.h = z.h.round();
    let gap = island_gap(self.dpi);
    if gap > 0.0 {
      z.radius = (z.h / 2.0).min(36.0 * self.dpi);
    }
    ISLAND_GAP.store(gap.to_bits(), std::sync::atomic::Ordering::Relaxed);
    self.body.set_island(gap, self.shown);
    if let Some(sh) = self.shadow.as_mut() {
      sh.set_island(gap, self.shown);
    }
    self.size = z;
    let x0 = self.x0();
    self.body.set_geometry(self.win, x0, z.w, z.h, z.radius, springy);
    let (blur, dy, op) = state::shadow(self.look);
    let over = if gap > 0.0 { 0.0 } else { self.body.top_over };
    if let Some(sh) = self.shadow.as_mut() {
      sh.follow(
        &self.paint.gfx,
        (x0, -over, z.w, z.h + over),
        z.radius,
        (blur * self.dpi, dy * self.dpi, op),
        springy,
      );
    }
    let e = self.body.ear;
    let (l, r) = (px0.min(x0) - e, (px0 + pw).max(x0 + z.w) + e);
    self.set_region((l.max(0.0), 0.0, r.min(self.win.0 as f32), gap + ph.max(z.h)));
    timer(self.hwnd, T_REGION, 700);
  }

  pub(super) fn set_region(&self, r: (f32, f32, f32, f32)) {
    unsafe {
      let rgn = CreateRectRgn(r.0 as i32, r.1 as i32, r.2.ceil() as i32, r.3.ceil() as i32);
      SetWindowRgn(self.hwnd, Some(rgn), true);
    }
  }

  pub(super) fn settle_region(&self) {
    let (x0, e) = (self.x0(), self.body.ear);
    self.set_region((
      (x0 - e).max(0.0),
      0.0,
      (x0 + self.size.w + e).min(self.win.0 as f32),
      island_gap(self.dpi) + self.size.h,
    ));
  }

  pub(super) fn render(&mut self) {
    let c = self.body.content.clone();
    if let Ok(k) = c.Children() {
      let _ = k.RemoveAll();
    }
    self.regions.clear();
    let f = Frame {
      x0: self.x0(),
      w: self.size.w,
      h: self.size.h,
    };
    let now_ms = self.now_ms();
    let items = self.glance_items();
    let item = items[self.glance_idx.min(items.len() - 1)];
    let glance = (self.look.view == View::Compact(state::Activity::Glance)).then(|| (item, self.glance_text(item)));
    self.live_vis = views::Live::default();
    if self.mode == Mode::Open(Tab::Shelf) {
      self.load_shelf_icons();
    }
    if self.mode == Mode::Open(Tab::Timer) {
      self.ensure_events();
    }
    if self.has_media && self.art.is_none() {
      self.ensure_app_icon();
    }
    let cal_date = self.cal_date();
    let events = match &self.cal_events {
      Some((d, Some(list))) if *d == cal_date => views::Events::List(list),
      Some((d, None)) if *d == cal_date => views::Events::Unavailable,
      _ => views::Events::Loading,
    };
    let send_c = self.send_compact();
    let cal = views::CalView {
      today: Self::today(),
      sel: cal_date,
      events,
    };
    let send = match &self.send {
      Some(ss) => {
        let n = ss.names.len();
        let title = if n == 1 {
          ss.names[0].clone()
        } else {
          format!("{} {}", crate::prefs::tr("Kirim"), views::count_text(n))
        };
        let frac = nexora_core::dynamic_island::send::fraction(ss.prog.sent, ss.prog.total);
        let done = ss.done_at.is_some() && !ss.prog.active;
        let status = if done {
          format!("{} \u{2713}", crate::prefs::tr("Terkirim"))
        } else if ss.prog.active || ss.prog.sent > 0 {
          format!("{} {}%", crate::prefs::tr("Mengirim"), (frac * 100.0).floor() as u32)
        } else {
          crate::prefs::tr("Menunggu HP\u{2026}").to_string()
        };
        Some(views::SendView {
          qr: ss.qr.as_ref(),
          qr_px: ss.qr_px,
          title,
          status,
          frac,
          done,
          error: false,
        })
      }
      None if self.send_err => Some(views::SendView {
        qr: None,
        qr_px: 0.0,
        title: String::new(),
        status: String::new(),
        frac: 0.0,
        done: false,
        error: true,
      }),
      None => None,
    };
    let app_icon = if self.art.is_none() {
      self.app_icons.get(&self.media_app).and_then(|s| s.as_ref())
    } else {
      None
    };
    let pos_ms = self.position_ms();
    let media = self.has_media.then(|| MediaView {
      title: &self.title,
      artist: &self.artist,
      playing: self.playing,
      pos_ms,
      dur_ms: self.dur_ms,
      can_seek: self.can_seek,
      art: self.art.as_ref().or(app_icon),
      tint: self.tint,
      drag: self.seek_drag.map(|(_, _, fr)| fr),
    });
    let lyrics = LyricsView {
      lines: self.lyr.as_ref(),
      state: self.lyr_state,
      pos_ms,
      lrclib: self.lyr_lrclib,
    };
    let shelf = self
      .shelf
      .items()
      .iter()
      .map(|i| (i.name(), self.shelf_icons.get(&i.path)))
      .collect();
    let drop_count = if self.mode == Mode::Drop {
      self.dnd.borrow().len()
    } else {
      0
    };
    let d = Data {
      battery: self.battery,
      timer: &self.timer,
      now_ms,
      glance,
      media,
      lyrics,
      shelf,
      drop_zone: self.drop_zone,
      drop_count,
      privacy: self.privacy,
      download: (self.look.view == View::Compact(state::Activity::Download))
        .then(|| Self::download_view(&self.dl, &self.dl_done, &self.dl_icons, send_c))
        .flatten(),
      cal,
      send,
      send_zone: crate::prefs::get().notch_send,
    };
    views::draw(
      &mut self.paint,
      &c,
      self.look,
      f,
      &d,
      &mut self.regions,
      &mut self.live_vis,
    );
    if let Some(r) = self.hover_act.and_then(|a| self.regions.iter().find(|r| r.action == a)) {
      if let Some(v) = &r.vis {
        let k = hover_scale(r.action);
        let _ = v.SetCenterPoint(Vector3 {
          X: r.x + r.w / 2.0,
          Y: r.y + r.h / 2.0,
          Z: 0.0,
        });
        let _ = v.SetScale(Vector3 { X: k, Y: k, Z: 1.0 });
      }
    }
    self.sync_visualizer();
    self.sync_progress();
  }

  pub(super) fn retire_content(&mut self, ms: u32) {
    let go = || -> windows_core::Result<()> {
      let kids = self.body.content.Children()?;
      let old: Vec<Visual> = kids.clone().into_iter().collect();
      let ghost = self.body.ghost.Children()?;
      ghost.RemoveAll()?;
      self.body.ghost.SetOffset(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 })?;
      for v in &old {
        kids.Remove(v)?;
        ghost.InsertAtTop(v)?;
      }
      let g: CompositionObject = self.body.ghost.cast()?;
      keyframes(&self.paint.comp, &g, "Opacity", 1.0, 0.0, ms, 0)
    };
    let _ = go();
    timer(self.hwnd, T_GHOST, 260);
  }

  pub(super) fn intro(&self, opening: bool) {
    let go = || -> windows_core::Result<()> {
      let c: CompositionObject = self.body.content.cast()?;
      let comp = &self.paint.comp;
      if opening {
        let cx = self.x0() + self.size.w / 2.0;
        self.body.content.SetCenterPoint(Vector3 { X: cx, Y: 0.0, Z: 0.0 })?;
        let (d, ms) = (motion::CONTENT_DELAY_MS, motion::CONTENT_FADE_MS);
        keyframes(comp, &c, "Opacity", 0.0, 1.0, ms, d)?;
        keyframes(comp, &c, "Scale.X", motion::CONTENT_SCALE_FROM, 1.0, ms + 60, d)?;
        keyframes(comp, &c, "Scale.Y", motion::CONTENT_SCALE_FROM, 1.0, ms + 60, d)?;
        keyframes(
          comp,
          &c,
          "Offset.Y",
          -motion::CONTENT_RISE_DP * self.dpi,
          0.0,
          ms + 60,
          d,
        )
      } else {
        self.body.content.SetScale(Vector3 { X: 1.0, Y: 1.0, Z: 1.0 })?;
        self.body.content.SetOffset(Vector3 { X: 0.0, Y: 0.0, Z: 0.0 })?;
        keyframes(comp, &c, "Opacity", 0.0, 1.0, 150, 50)
      }
    };
    let _ = go();
  }
}
