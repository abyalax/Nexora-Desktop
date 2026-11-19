//! Mouse input: press, drag, hover, click, wheel, context menu.

use super::*;

impl Notch {
    pub(super) fn on_down(&mut self, x: f32, y: f32) -> bool {
        if !matches!(self.mode, Mode::Open(_)) {
            return false;
        }
        if let Some(Action::Seek(sx, sw)) =
            self.regions.iter().find(|r| r.hit(x, y)).map(|r| r.action)
        {
            self.seek_drag = Some((sx, sw, ((x - sx) / sw).clamp(0.0, 1.0)));
            self.render();
            return true;
        }
        false
    }

    pub(super) fn on_drag(&mut self, x: f32) {
        if let Some((sx, sw, _)) = self.seek_drag {
            self.seek_drag = Some((sx, sw, ((x - sx) / sw).clamp(0.0, 1.0)));
            self.render();
        }
    }

    pub(super) fn on_up(&mut self) -> bool {
        let Some((_, _, fr)) = self.seek_drag.take() else {
            return false;
        };
        if self.dur_ms > 0 {
            let ms = (self.dur_ms as f32 * fr) as i64;
            let _ = self.media.commands.send(Command::SeekMs(ms));
            self.pos_ms = ms;
            self.pos_at = Instant::now();
        }
        self.render();
        self.schedule_lyric();
        true
    }

    pub(super) fn on_right_click(&mut self, x: f32, y: f32) {
        if self.mode != Mode::Open(Tab::Shelf) {
            return;
        }
        if let Some(Action::ShelfItem(i)) =
            self.regions.iter().find(|r| r.hit(x, y)).map(|r| r.action)
        {
            self.shelf_menu(i, x, y);
        }
    }

    pub(super) fn on_move(&mut self) {
        if !self.tracking {
            let mut t = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE,
                hwndTrack: self.hwnd,
                dwHoverTime: 0,
            };
            if unsafe { TrackMouseEvent(&mut t) }.is_ok() {
                self.tracking = true;
            }
        }
        kill(self.hwnd, T_LEAVE);
        if self.grace {
            self.grace = false;
            kill(self.hwnd, T_GRACE);
        }
        self.dispatch(Event::Enter);
        if self.mode == Mode::Peek && crate::prefs::get().notch_hover_open {
            if self.cursor_on_media_button() {
                kill(self.hwnd, T_DWELL);
            } else {
                timer(self.hwnd, T_DWELL, motion::DWELL_MS);
            }
        }
    }

    pub(super) fn on_hover(&mut self, x: f32, y: f32) {
        let hit = self
            .regions
            .iter()
            .find(|r| r.vis.is_some() && r.hit(x, y))
            .map(|r| r.action);
        if hit == self.hover_act {
            return;
        }
        for (a, on) in [(self.hover_act, false), (hit, true)] {
            let Some(r) = a.and_then(|a| self.regions.iter().find(|r| r.action == a)) else {
                continue;
            };
            let Some(v) = &r.vis else { continue };
            let to = if on { hover_scale(r.action) } else { 1.0 };
            let Ok(comp) = v.Compositor() else { continue };
            let _ = crate::motion::hover_pop(
                &comp,
                v,
                Vector3 {
                    X: r.x + r.w / 2.0,
                    Y: r.y + r.h / 2.0,
                    Z: 0.0,
                },
                to,
            );
        }
        self.hover_act = hit;
    }

    pub(super) fn on_leave(&mut self) {
        self.on_hover(-1.0, -1.0);
        self.tracking = false;
        kill(self.hwnd, T_DWELL);
        if self.seek_drag.is_some() || self.menu.as_ref().is_some_and(|m| m.is_open()) {
            return;
        }
        if matches!(self.mode, Mode::Open(_)) {
            timer(self.hwnd, T_LEAVE, motion::LEAVE_GRACE_MS);
        } else {
            self.dispatch(Event::Leave);
        }
        self.grace = true;
        timer(self.hwnd, T_GRACE, 800);
        self.update_visibility();
    }

    pub(super) fn on_media_button(&self, x: f32, y: f32) -> bool {
        self.look.view == View::Compact(state::Activity::Player)
            && self.regions.iter().any(|r| {
                r.hit(x, y) && matches!(r.action, Action::PlayPause | Action::Next | Action::Prev)
            })
    }

    pub(super) fn cursor_on_media_button(&self) -> bool {
        let mut pt = windows::Win32::Foundation::POINT::default();
        unsafe {
            if GetCursorPos(&mut pt).is_err() {
                return false;
            }
            let _ = windows::Win32::Graphics::Gdi::ScreenToClient(self.hwnd, &mut pt);
        }
        self.on_media_button(pt.x as f32, pt.y as f32)
    }

    pub(super) fn on_click(&mut self, x: f32, y: f32) {
        if !matches!(self.mode, Mode::Open(_)) && !self.on_media_button(x, y) {
            kill(self.hwnd, T_DWELL);
            self.dispatch(Event::Click);
            return;
        }
        let Some(a) = self.regions.iter().find(|r| r.hit(x, y)).map(|r| r.action) else {
            return;
        };
        let now = self.now_ms();
        match a {
            Action::PlayPause => {
                let _ = self.media.commands.send(Command::PlayPause);
            }
            Action::Next => {
                let _ = self.media.commands.send(Command::Next);
            }
            Action::Prev => {
                let _ = self.media.commands.send(Command::Prev);
            }
            Action::Seek(..) | Action::Zone(_) => {}
            Action::ShelfItem(i) => {
                if let Some(it) = self.shelf.items().get(i) {
                    nexora_platform::launch::open(&it.path);
                }
            }
            Action::ShelfClear => {
                self.shelf.clear();
                self.shelf_icons.clear();
                self.refresh();
            }
            Action::Tab(t) => self.dispatch(Event::SelectTab(t)),
            Action::Settings => crate::settings_win::open(&self.paint.comp),
            Action::TimerToggle => {
                if self.timer.phase == Phase::Running {
                    self.timer.pause(now);
                } else {
                    self.timer.start(now);
                }
                self.sync_tick();
                self.refresh();
            }
            Action::TimerReset => {
                self.timer.reset();
                self.sync_tick();
                self.refresh();
            }
            Action::TimerPlus => {
                self.timer.adjust(300);
                self.refresh();
            }
            Action::TimerMinus => {
                self.timer.adjust(-300);
                self.refresh();
            }
            Action::SendDone => self.stop_send(),
            Action::CalDay(n) => {
                self.cal_sel = (self.cal_sel + n).clamp(-3650, 3650);
                self.render();
            }
        }
    }

    pub(super) fn on_wheel(&mut self, delta: i16) {
        if let Mode::Open(cur) = self.mode {
            let i = Tab::ALL.iter().position(|t| *t == cur).unwrap_or(0) as i32;
            let n = Tab::ALL.len() as i32;
            let next = Tab::ALL[(i + if delta > 0 { -1 } else { 1 }).rem_euclid(n) as usize];
            self.dispatch(Event::SelectTab(next));
        }
    }
}
