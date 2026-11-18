//! Finder bar item layout, application label and clock text.

use super::*;

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
        let native = nacre_platform::menus::read(fg).unwrap_or_default();
        let titles = |v: &Vec<nacre_platform::menus::Top>| {
            v.iter().map(|t| t.title.clone()).collect::<Vec<_>>()
        };
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
                    nacre_core::dock::items::name_from_path(&w.path)
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
        let pad = 11.0 * d;
        let t = |s: &str, wgt: i32| self.text_w(s, wgt) + 2.0 * pad;
        let mut left = vec![
            LItem {
                id: LOGO,
                width: self.h as f32 * 1.05,
            },
            LItem {
                id: APP,
                width: t(&self.app_label(), APP_W),
            },
        ];
        if self.native.is_empty() {
            left.push(LItem {
                id: M_FILE,
                width: t(&mt("Berkas"), BAR_W),
            });
            left.push(LItem {
                id: M_EDIT,
                width: t(&mt("Edit"), BAR_W),
            });
            left.push(LItem {
                id: M_VIEW,
                width: t(&mt("Menu:Tampilan"), BAR_W),
            });
            left.push(LItem {
                id: M_WIN,
                width: t(&mt("Menu:Jendela"), BAR_W),
            });
            left.push(LItem {
                id: M_HELP,
                width: t(&mt("Bantuan"), BAR_W),
            });
        } else {
            for (i, top) in self.native.iter().enumerate().take(20) {
                left.push(LItem {
                    id: NATIVE0 + i as u32,
                    width: t(&top.title, BAR_W),
                });
            }
        }
        let icon_w = 30.0 * d;
        let mut right = vec![
            LItem {
                id: CLOCK,
                width: t(&self.clock_text(), BAR_W),
            },
            LItem {
                id: CTRL,
                width: icon_w,
            },
            LItem {
                id: SEARCH,
                width: icon_w,
            },
        ];
        if prefs::get().stage {
            right.push(LItem {
                id: STAGEI,
                width: icon_w,
            });
        }
        if self.wifi.is_some() {
            right.push(LItem {
                id: WIFI,
                width: icon_w,
            });
        }
        if let Some((pct, _)) = sys::battery() {
            let extra = if prefs::get().battery_pct == 2 {
                self.small_w(&format!("{pct}%"), 12.0, BAR_W) + 5.0 * d
            } else {
                0.0
            };
            right.push(LItem {
                id: BATT,
                width: 40.0 * d + extra,
            });
        }
        let gap = services::notch().map(|(l, r, _)| Gap {
            left: l as f32,
            right: r as f32,
        });
        let (l, r) = lay::layout(self.screen.0 as f32, 6.0 * d, &left, &right, gap);
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
            "NEXORA".to_string()
        } else {
            self.app.clone()
        }
    }

    pub(super) fn clock_text(&self) -> String {
        let n = time::local_now();
        let when = nacre_core::finder::clock::When {
            year: n.year,
            month: n.month,
            day: n.day,
            hour: n.hour,
            minute: n.minute,
        };
        nacre_core::finder::clock::format(when, prefs::get().lang, prefs::get().clock_24h)
    }
}
