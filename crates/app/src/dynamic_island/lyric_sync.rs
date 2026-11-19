//! Lyrics lookup and per-line synchronisation with playback.

use super::*;

impl Notch {
    pub(super) fn request_lyrics(&mut self) {
        self.lyr = None;
        self.lyr_idx = None;
        self.lyr_lrclib = false;
        if !self.has_media {
            self.lyr_state = LyrState::Idle;
            self.lyr_key.clear();
            return;
        }
        let (title, artist) = nexora_core::dynamic_island::lrclib::clean(&self.title, &self.artist);
        let track = nexora_core::dynamic_island::lrclib::Track {
            title,
            artist,
            album: self.album.clone(),
            duration_s: (self.dur_ms > 0).then(|| ((self.dur_ms + 500) / 1000) as u32),
        };
        self.lyr_key = nexora_core::dynamic_island::lrclib::cache_key(&track);
        self.lyr_state = LyrState::Loading;
        let online = crate::prefs::get().lyrics_online;
        let _ = self
            .lyr_worker
            .requests
            .send(nexora_platform::lyrics::Request { track, online });
    }

    pub(super) fn on_lyrics(&mut self) {
        let mut got = false;
        while let Ok(a) = self.lyr_worker.answers.try_recv() {
            if a.key != self.lyr_key {
                continue;
            }
            got = true;
            let parsed = a
                .text
                .map(|t| nexora_core::dynamic_island::lyrics::parse(&t))
                .filter(|l| !l.is_empty());
            self.lyr_state = if parsed.is_some() {
                LyrState::Ready
            } else {
                LyrState::Missing
            };
            self.lyr_lrclib = a.source == nexora_platform::lyrics::Source::Lrclib;
            log::line(
                "lirik",
                &format!(
                    "{:?} sumber={:?} baris={}",
                    self.lyr_state,
                    a.source,
                    parsed.as_ref().map_or(0, |l| l.lines.len())
                ),
            );
            self.lyr = parsed;
            self.lyr_idx = None;
        }
        if got {
            self.refresh();
            self.schedule_lyric();
        }
    }

    pub(super) fn schedule_lyric(&self) {
        kill(self.hwnd, T_LYRIC);
        let Some(l) = &self.lyr else { return };
        if !l.synced || !self.playing {
            return;
        }
        let pos = self.position_ms();
        if let Some(next) = l.next_change_ms(pos) {
            timer(
                self.hwnd,
                T_LYRIC,
                ((next - pos).max(10) + 15).min(60_000) as u32,
            );
        }
    }

    pub(super) fn on_lyric_tick(&mut self) {
        let pos = self.position_ms();
        let idx = self.lyr.as_ref().and_then(|l| l.index_at(pos));
        if idx != self.lyr_idx {
            let forward = matches!((self.lyr_idx, idx), (Some(a), Some(b)) if b == a + 1)
                || (self.lyr_idx.is_none() && idx == Some(0));
            self.lyr_idx = idx;
            if self.look.view == View::Compact(state::Activity::Lyrics) {
                self.swap_lyric();
            } else if self.mode == Mode::Open(Tab::Play)
                || self.look.view == View::Compact(state::Activity::Player)
            {
                self.render();
                if forward {
                    if let Some(sc) = &self.live_vis.lyric_scroll {
                        if let Ok(o) = sc.cast::<CompositionObject>() {
                            let _ = keyframes(
                                &self.paint.comp,
                                &o,
                                "Offset.Y",
                                self.live_vis.lyric_shift,
                                0.0,
                                420,
                                0,
                            );
                        }
                    }
                }
            }
        }
        self.schedule_lyric();
    }

    pub(super) fn swap_lyric(&mut self) {
        let rise = state::motion::LYRIC_RISE_DP * self.dpi;
        let ms = state::motion::LYRIC_SWAP_MS;
        if let Some(old) = self.live_vis.lyric.take() {
            let go = || -> windows_core::Result<()> {
                let ghost = self.body.ghost.Children()?;
                ghost.RemoveAll()?;
                self.body.ghost.SetOffset(Vector3 {
                    X: 0.0,
                    Y: 0.0,
                    Z: 0.0,
                })?;
                self.body.content.Children()?.Remove(&old)?;
                ghost.InsertAtTop(&old)?;
                let g: CompositionObject = self.body.ghost.cast()?;
                keyframes(&self.paint.comp, &g, "Opacity", 1.0, 0.0, ms, 0)?;
                keyframes(&self.paint.comp, &g, "Offset.Y", 0.0, -rise, ms, 0)
            };
            let _ = go();
            timer(self.hwnd, T_GHOST, ms + 60);
        }
        self.render();
        if let Some(v) = &self.live_vis.lyric {
            if let (Ok(o), Ok(at)) = (v.cast::<CompositionObject>(), v.Offset()) {
                let comp = &self.paint.comp;
                let _ = keyframes(comp, &o, "Opacity", 0.0, 1.0, ms, 40);
                let _ = keyframes(comp, &o, "Offset.Y", at.Y + rise, at.Y, ms + 40, 0);
            }
        }
    }
}
