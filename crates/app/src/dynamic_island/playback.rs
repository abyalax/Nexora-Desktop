//! Media session state, progress and the audio visualizer.

use super::*;

impl Notch {
  pub(super) fn position_ms(&self) -> i64 {
    let extra = if self.playing {
      self.pos_at.elapsed().as_millis() as i64
    } else {
      0
    };
    let p = self.pos_ms + extra;
    if self.dur_ms > 0 {
      p.clamp(0, self.dur_ms)
    } else {
      p.max(0)
    }
  }

  pub(super) fn on_media(&mut self) {
    let mut changed = false;
    while let Ok(s) = self.media.snapshots.try_recv() {
      changed = true;
      self.has_media = s.has_media;
      self.title = s.title;
      self.artist = s.artist;
      self.album = s.album;
      self.media_app = s.app;
      self.playing = s.playing;
      self.pos_ms = s.position_ms;
      self.dur_ms = s.duration_ms;
      self.pos_at = Instant::now();
      self.can_seek = s.can_seek;
      match s.art {
        ArtUpdate::Unchanged => {}
        ArtUpdate::None => {
          self.art = None;
          self.tint = None;
        }
        ArtUpdate::New(b) => {
          self.tint = nexora_core::dynamic_island::tint::from_art(&b.bgra, b.w, b.h);
          self.art = self.paint.gfx.surface_from_bgra(b.w, b.h, &b.bgra).ok();
        }
      }
    }
    if changed {
      let track = if self.has_media {
        format!("{}\u{1}{}", self.title, self.artist)
      } else {
        String::new()
      };
      if track != self.lyr_track {
        self.lyr_track = track;
        self.request_lyrics();
      }
      self.refresh();
      self.schedule_lyric();
    }
  }

  pub(super) fn sync_visualizer(&mut self) {
    let want = self.has_media && self.playing && self.shown && !self.overlap.fullscreen && !self.off;
    if want && self.loopback.is_none() {
      let hw = self.hwnd.0 as isize;
      self.loopback = Some(loopback::start(move || post(hw, WM_BANDS)));
    } else if !want && self.loopback.is_some() {
      self.loopback = None;
    }
  }

  pub(super) fn on_bands(&mut self) {
    let b = loopback::latest();
    for (i, bar) in self.live_vis.bars.iter().enumerate() {
      let v = b.get(i).copied().unwrap_or(0.0).clamp(0.0, 1.0);
      let _ = bar.SetScale(Vector3 {
        X: 1.0,
        Y: 0.25 + 0.75 * v,
        Z: 1.0,
      });
    }
  }

  pub(super) fn sync_progress(&self) {
    if self.mode == Mode::Open(Tab::Play) && self.has_media && self.playing && self.seek_drag.is_none() {
      timer(self.hwnd, T_PROGRESS, 1000);
    } else {
      kill(self.hwnd, T_PROGRESS);
    }
  }
}
