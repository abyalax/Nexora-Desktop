//! Periodic ticks and the WM_TIMER dispatcher.

use super::*;

impl Notch {
    pub(super) fn sync_tick(&self) {
        if self.timer.phase == Phase::Running {
            timer(self.hwnd, T_TICK, 250);
        } else {
            kill(self.hwnd, T_TICK);
        }
    }

    pub(super) fn on_tick(&mut self) {
        let now = self.now_ms();
        let finished = self.timer.tick(now);
        let s = self.timer.remaining_secs(now);
        if finished || s != self.last_sec {
            self.last_sec = s;
            self.refresh();
        }
        if finished {
            log::line("notch", "timer selesai");
            self.sync_tick();
        }
    }

    pub(super) fn on_timer(&mut self, id: usize) {
        if id != T_TICK {
            kill(self.hwnd, id);
        }
        match id {
            T_DWELL => {
                if self.tracking && !self.cursor_on_media_button() {
                    self.dispatch(Event::Dwell);
                }
            }
            T_LEAVE => {
                if !self.tracking && !self.send_pinned() {
                    self.dispatch(Event::Leave);
                }
            }
            T_GHOST => {
                if let Ok(k) = self.body.ghost.Children() {
                    let _ = k.RemoveAll();
                }
            }
            T_REGION => self.settle_region(),
            T_TICK => self.on_tick(),
            T_CHARGE => {
                self.charging_fresh = false;
                self.refresh();
            }
            T_WINCHG => {
                let mut ov = overlap::evaluate(
                    tracker::external_foreground(),
                    std::process::id(),
                    self.pill_screen_rect(),
                    sys::screen_size(),
                );
                if ov.fullscreen && crate::prefs::get().above_fullscreen {
                    ov = overlap::Overlap {
                        overlapped: false,
                        fullscreen: false,
                    };
                }
                if ov != self.overlap {
                    self.overlap = ov;
                    self.update_visibility();
                    self.sync_visualizer();
                }
            }
            T_RECONF => self.on_reconfigure(),
            T_EDGE => {
                self.edge_armed = true;
                self.update_visibility();
            }
            T_GRACE => {
                self.grace = false;
                self.update_visibility();
            }
            T_START => {
                self.starting = false;
                self.update_visibility();
            }
            T_GLANCE => {
                self.rotate_glance();
                self.sync_glance();
            }
            T_LYRIC => self.on_lyric_tick(),
            T_PRIVACY => self.poll_privacy(),
            T_SEND => {
                let idle = self
                    .send
                    .as_ref()
                    .is_some_and(|s| !s.prog.active && s.last.elapsed() > SEND_IDLE);
                if idle {
                    log::line("kirim", "sesi menganggur 15 menit");
                    self.stop_send();
                } else if self.send.is_some() {
                    timer(self.hwnd, T_SEND, 30_000);
                }
            }
            T_SENDDONE => {
                kill(self.hwnd, T_SENDDONE);
                self.refresh();
            }
            T_TESTSEND => {
                kill(self.hwnd, T_TESTSEND);
                if let Ok(p) = std::env::var("NACRE_TEST_SEND") {
                    self.start_send(p.split(';').map(|s| s.to_string()).collect());
                }
            }
            T_DLDONE => {
                unsafe {
                    let _ = KillTimer(Some(self.hwnd), T_DLDONE);
                }
                self.dl_done = None;
                self.refresh();
            }
            T_PROGRESS => {
                if self.mode == Mode::Open(Tab::Play) {
                    self.render();
                }
            }
            T_MINUTE => {
                if self.look.view == View::Compact(state::Activity::Glance) {
                    self.render();
                }
                timer(
                    self.hwnd,
                    T_MINUTE,
                    nexora_platform::time::ms_to_next_minute(),
                );
            }
            T_HIDEWIN => {
                if !self.shown {
                    unsafe {
                        let _ = ShowWindow(self.hwnd, SW_HIDE);
                    }
                    if let Some(sh) = &self.shadow {
                        sh.set_visible(false);
                    }
                }
            }
            _ => {}
        }
    }
}
