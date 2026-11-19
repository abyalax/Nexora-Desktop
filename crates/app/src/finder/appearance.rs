//! Finder bar colors, background, shade and fullscreen tracking.

use super::*;

impl Bar {
    pub(super) fn fg(&self) -> [u8; 3] {
        if self.shadow {
            [0, 0, 0]
        } else if self.dark_text {
            [18, 18, 22]
        } else {
            [255, 255, 255]
        }
    }

    pub(super) fn apply_bg(&mut self) {
        use nexora_core::settings::BarStyle;
        let p = prefs::get();
        let d = self.dpi;
        let flat = if p.menubar_glass && !p.menubar_material {
            Some(match p.menubar_style {
                BarStyle::Light => (248, 248, 250),
                BarStyle::Dark => (0, 0, 0),
            })
        } else {
            None
        };
        if p.menubar_material {
            let look = prefs::look(prefs::Role::Themed);
            let base =
                nexora_ui::glass::GlassParams::dark((self.screen.0 as f32, self.h as f32), 0.0);
            let mut gp = prefs::apply(base, &look);
            gp.rim.A = 0;
            gp.highlight = 0.0;
            let _ = self.popup.panel.set_fringe(false);
            let _ = self.popup.panel.set_params(gp);
            let _ = self
                .popup
                .panel
                .set_rect(0.0, 0.0, self.screen.0 as f32, self.h as f32);
        } else {
            let _ = self.popup.panel.set_rect(0.0, 0.0, 0.0, 0.0);
        }
        let _ = self.bg.SetSize(Vector2 {
            X: self.screen.0 as f32,
            Y: self.h as f32,
        });
        match flat {
            Some((r, g, b)) => {
                if let Ok(br) = self.comp.CreateColorBrushWithColor(Color {
                    A: 255,
                    R: r,
                    G: g,
                    B: b,
                }) {
                    let _ = self.bg.SetBrush(&br);
                }
                let _ = self.bg.SetOpacity(1.0);
            }
            None if !p.menubar_material => {
                if let Ok(br) = self.top_shade() {
                    let _ = self.bg.SetBrush(&br);
                    let _ = self.bg.SetOpacity(1.0);
                }
            }
            None => {
                let _ = self.bg.SetOpacity(0.0);
            }
        }
        let line = d.round().max(1.0);
        let _ = self.hair.SetSize(Vector2 {
            X: self.screen.0 as f32,
            Y: line,
        });
        let _ = self.hair.SetOffset(Vector3 {
            X: 0.0,
            Y: self.h as f32 - line,
            Z: 0.0,
        });
        let hc = match (p.menubar_glass && !p.menubar_material, p.menubar_style) {
            _ if p.menubar_material => Some(if prefs::look(prefs::Role::Themed).dark {
                Color {
                    A: 18,
                    R: 255,
                    G: 255,
                    B: 255,
                }
            } else {
                Color {
                    A: 22,
                    R: 0,
                    G: 0,
                    B: 0,
                }
            }),
            (false, _) => None,
            (true, BarStyle::Light) => Some(Color {
                A: 34,
                R: 0,
                G: 0,
                B: 0,
            }),
            (true, BarStyle::Dark) => Some(Color {
                A: 40,
                R: 255,
                G: 255,
                B: 255,
            }),
        };
        match hc.and_then(|c| self.comp.CreateColorBrushWithColor(c).ok()) {
            Some(br) => {
                let _ = self.hair.SetBrush(&br);
                let _ = self.hair.SetOpacity(1.0);
            }
            None => {
                let _ = self.hair.SetOpacity(0.0);
            }
        }
    }

    pub(super) fn top_shade(
        &self,
    ) -> windows_core::Result<windows::UI::Composition::CompositionLinearGradientBrush> {
        let g = self.comp.CreateLinearGradientBrush()?;
        g.SetStartPoint(Vector2 { X: 0.0, Y: 0.0 })?;
        g.SetEndPoint(Vector2 { X: 0.0, Y: 1.0 })?;
        let stops = g.ColorStops()?;
        for (o, a) in [(0.0, 40u8), (0.45, 18), (1.0, 0)] {
            stops.Append(&self.comp.CreateColorGradientStopWithOffsetAndColor(
                o,
                Color {
                    A: a,
                    R: 0,
                    G: 0,
                    B: 0,
                },
            )?)?;
        }
        Ok(g)
    }

    pub(super) fn update_text_color(&mut self) -> bool {
        use nexora_core::settings::BarStyle;
        let before = self.dark_text;
        let p = prefs::get();
        self.dark_text =
            p.menubar_glass && !p.menubar_material && p.menubar_style == BarStyle::Light;
        before != self.dark_text
    }

    pub(super) fn recheck_color(&mut self) {
        if self.update_text_color() {
            self.cache.clear();
            self.render();
        }
    }

    pub(super) fn apply_look(&mut self) {
        self.update_text_color();
        self.apply_bg();
        self.cache.clear();
    }

    pub(super) fn check_fullscreen(&mut self) {
        let fg = focus::foreground();
        if fg == 0 || fg == self.popup.hwnd.0 as isize {
            return;
        }
        let full = is_fullscreen(fg) && !prefs::get().above_fullscreen;
        if full != self.fullscreen {
            self.fullscreen = full;
            if full {
                self.close_menu();
                self.popup.hide();
            } else {
                self.popup.show();
                crate::topmost::request_raise();
            }
        }
    }
}
