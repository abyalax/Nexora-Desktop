//! Item views, icon surfaces and preference-driven re-rendering.

use super::*;

impl Dock {
    pub fn refresh_windows(&mut self) {
        let wins = enumerate::list(std::process::id());
        tracker::prune(focus::is_alive);
        let mut items = merge(&self.pins, &wins, &tracker::recent, &mut self.seen);
        self.heal_pins(&mut items);
        self.track_recents(&items);
        self.append_recents(&mut items);
        let sig: Vec<_> = items
            .iter()
            .map(|i| (i.key.clone(), i.pinned, i.hwnds.len(), i.path.clone()))
            .collect();
        self.items = items;
        if sig != self.signature {
            self.signature = sig;
            self.rebuild();
        }
        self.overlap = overlap::evaluate(
            tracker::external_foreground(),
            std::process::id(),
            self.card_screen_rect(),
            self.screen,
        );
        if self.overlap.fullscreen && crate::prefs::get().above_fullscreen {
            self.overlap = overlap::Overlap {
                overlapped: false,
                fullscreen: false,
            };
        }
        self.update_visibility();
    }

    pub(super) fn rebuild(&mut self) {
        log::line(
            "dock",
            &format!(
                "item: {}",
                self.items
                    .iter()
                    .map(|i| format!(
                        "{}[{}]{}",
                        i.name,
                        i.hwnds.len(),
                        if i.pinned { "*" } else { "" }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
        let show_downloads = crate::prefs::get().downloads_stack && nacre_platform::downloads::folder().is_some();
        self.entries = entries::build(&self.items, show_downloads);
        self.slots = entries::slots(&self.entries);

        if let Ok(kids) = self.content.Children() {
            let _ = kids.RemoveAll();
        }
        self.views.clear();
        let icon = self.m.icon;
        if !icons::is_themed(LAUNCHPAD_KEY) && !self.surfaces.contains_key(LAUNCHPAD_KEY) {
            let px = ((self.m.icon * self.m.max_scale * 1.15) as i32).clamp(64, 256) as u32;
            let p = crate::prefs::get();
            let data = nacre_core::iconplate::style_icon_v(
                &nacre_core::iconplate::launchpad_icon(px),
                p.icon_style,
                p.icon_tint,
                crate::prefs::icon_dark(),
            );
            if let Ok(s) = self.gfx.surface_from_bgra(px, px, &data) {
                self.surfaces.insert(LAUNCHPAD_KEY.to_string(), s);
            }
        }
        if !icons::is_themed(TRASH_PATH) && !self.surfaces.contains_key(TRASH_PATH) {
            self.make_trash_surface();
        }
        let look = Appearance {
            dark: crate::prefs::look(crate::prefs::Role::Dock).dark,
            plates: crate::prefs::get().dock_plates,
        };
        for e in self.entries.clone() {
            let v = match e {
                Entry::Start if custom_icons::has(START_KEY) => View::app(
                    &self.comp,
                    &self.content,
                    START_KEY.to_string(),
                    icon,
                    false,
                    self.dpi,
                    look,
                ),
                Entry::Start => View::start(&self.comp, &self.content, icon, look),
                Entry::Launchpad => View::app(
                    &self.comp,
                    &self.content,
                    LAUNCHPAD_KEY.to_string(),
                    icon,
                    false,
                    self.dpi,
                    look,
                ),
                Entry::Sep => View::separator(&self.comp, &self.content, self.m.separator, icon),
                Entry::Trash => View::app(
                    &self.comp,
                    &self.content,
                    TRASH_PATH.to_string(),
                    icon,
                    false,
                    self.dpi,
                    look,
                ),
                Entry::Downloads => View::app(
                    &self.comp,
                    &self.content,
                    downloads_key(),
                    icon,
                    false,
                    self.dpi,
                    look,
                ),
                Entry::App(i) => {
                    let it = &self.items[i];
                    View::app(
                        &self.comp,
                        &self.content,
                        it.path.clone(),
                        icon,
                        it.running(),
                        self.dpi,
                        look,
                    )
                }
            };
            match v {
                Ok(mut v) => {
                    if let Kind::App { path } = &v.kind {
                        if let Some(s) = self.surfaces.get(path) {
                            let _ = v.set_surface(&self.comp, s);
                        } else if self.requested.insert(path.clone()) {
                            let _ = self.worker.requests.send(path.clone());
                        }
                    }
                    self.views.push(v);
                }
                Err(e) => log::line("dock", &format!("gagal membuat view: {e}")),
            }
        }
        if self.views.len() != self.entries.len() {
            self.entries.truncate(self.views.len());
            self.slots.truncate(self.views.len());
        }
        self.hover = None;
        self.resize_window();
        self.apply(None, false);
        self.bind_expressions();
        self.set_region(self.tracking);
    }

    pub(super) fn bind_expressions(&mut self) {
        self.exprs_ok = false;
        self.k_on = false;
        let _ = self.props.InsertScalar(&HSTRING::from("k"), 0.0);
        if self.views.is_empty() {
            return;
        }
        let r = (|| -> windows_core::Result<()> {
            let n = self.views.len();
            let (icon, gap, sep) = (self.m.icon, self.m.gap, self.m.separator);
            let area = self.win.0 as f32;
            let rest = layout(&self.slots, None, area, &self.m);
            let is_item = |i: usize| self.slots[i] == Slot::Item;
            let nodes: Vec<CompositionObject> = self
                .views
                .iter()
                .map(|v| v.node.cast::<CompositionObject>())
                .collect::<Result<_, _>>()?;
            let props: CompositionObject = self.props.cast()?;
            let make = |text: &str| -> windows_core::Result<ExpressionAnimation> {
                let e = self
                    .comp
                    .CreateExpressionAnimationWithExpression(&HSTRING::from(text))?;
                e.SetReferenceParameter(&HSTRING::from("p"), &props)?;
                for (j, node) in nodes.iter().enumerate() {
                    if is_item(j) {
                        e.SetReferenceParameter(&HSTRING::from(format!("n{j}")), node)?;
                    }
                }
                Ok(e)
            };
            let const_total = self.slots.iter().filter(|s| **s == Slot::Separator).count() as f32
                * sep
                + gap * (n as f32 - 1.0);
            let mut total = format!("{const_total:.3}");
            for j in 0..n {
                if is_item(j) {
                    total += &format!(" + {icon:.3}*n{j}.Scale.X");
                }
            }
            let start = format!("({area:.3} - ({total}))/2");
            let reach = self.m.reach.max(1.0);
            let amp = self.m.max_scale - 1.0;
            for i in 0..n {
                let seps_before = (0..i).filter(|j| !is_item(*j)).count() as f32;
                let mut prefix = format!("{:.3}", seps_before * sep + gap * i as f32);
                for j in 0..i {
                    if is_item(j) {
                        prefix += &format!(" + {icon:.3}*n{j}.Scale.X");
                    }
                }
                let (w_i, base_w) = if is_item(i) {
                    (format!("{icon:.3}*n{i}.Scale.X"), icon)
                } else {
                    (format!("{sep:.3}"), sep)
                };
                let box_x = format!("({start} + ({prefix}))");
                let left = make(&format!("{box_x} + ({w_i})/2 - {:.3}", base_w / 2.0))?;
                nodes[i].StartAnimation(&HSTRING::from("Offset.X"), &left)?;
                if is_item(i) {
                    let c = rest.cells[i].x + rest.cells[i].w / 2.0;
                    let scale = make(&format!(
                        "1 + {amp:.4}*p.k*0.5*(1 + Cos(3.14159265*Clamp(Abs(p.cx - {c:.3})/{reach:.3}, 0, 1)))"
                    ))?;
                    nodes[i].StartAnimation(&HSTRING::from("Scale.X"), &scale)?;
                    nodes[i].StartAnimation(&HSTRING::from("Scale.Y"), &scale)?;
                }
                if let Some(dot) = &self.views[i].dot {
                    let half = dot.Size()?.X / 2.0;
                    let dx = make(&format!("{box_x} + ({w_i})/2 - {half:.3}"))?;
                    let d: CompositionObject = dot.cast()?;
                    d.StartAnimation(&HSTRING::from("Offset.X"), &dx)?;
                }
            }
            let card_w = format!("({total}) + {:.3}", 2.0 * self.pad_side);
            self.panel
                .bind_x_width(&make, &format!("({area:.3} - ({card_w}))/2"), &card_w)?;
            Ok(())
        })();
        match r {
            Ok(()) => self.exprs_ok = true,
            Err(e) => log::line(
                "dock",
                &format!("ekspresi magnifikasi gagal, memakai jalur animasi per ikon: {e}"),
            ),
        }
    }

    pub(super) fn set_influence(&mut self, on: bool) {
        if self.k_on == on {
            return;
        }
        self.k_on = on;
        if let Ok(v) = f32_ref(if on { 1.0 } else { 0.0 }) {
            if self.k_anim.SetFinalValue(&v).is_ok() {
                let o: Result<CompositionObject, _> = self.props.cast();
                if let Ok(o) = o {
                    let _ = o.StartAnimation(&HSTRING::from("k"), &self.k_anim);
                }
            }
        }
    }

    pub(super) fn apply(&mut self, cursor: Option<f32>, animate: bool) {
        let l = layout(&self.slots, cursor, self.win.0 as f32, &self.m);
        let card_w = l.total + 2.0 * self.pad_side;
        let cx = (self.win.0 as f32 - card_w) / 2.0;
        let card = (cx, self.card_y, card_w, self.card_h);
        let moved = |a: f32, b: f32| (a - b).abs() >= 0.5;
        if moved(card.0, self.last_card.0)
            || moved(card.2, self.last_card.2)
            || moved(card.1, self.last_card.1)
            || moved(card.3, self.last_card.3)
        {
            let _ = self.panel.set_rect(card.0, card.1, card.2, card.3);
            self.last_card = card;
        }
        for (i, (v, c)) in self.views.iter().zip(l.cells.iter()).enumerate() {
            if self.dragging == Some(i) {
                continue;
            }
            let _ = v.place(c.x, c.w, c.scale, self.icon_y, self.dot_y, animate);
        }
        self.layout = l;
        self.last_apply = Instant::now();
    }

    pub(super) fn make_trash_surface(&mut self) {
        let px = ((self.m.icon * self.m.max_scale * 1.15) as i32).clamp(64, 256) as u32;
        let p = crate::prefs::get();
        let raw = nacre_core::iconplate::free(&nacre_core::trash::icon(px, self.trash_full), px);
        let data = nacre_core::iconplate::style_icon_v(
            &raw,
            p.icon_style,
            p.icon_tint,
            crate::prefs::icon_dark(),
        );
        if let Ok(s) = self.gfx.surface_from_bgra(px, px, &data) {
            self.surfaces.insert(TRASH_PATH.to_string(), s);
        }
    }

    pub(super) fn update_trash_icon(&mut self) {
        if icons::is_themed(TRASH_PATH) {
            return;
        }
        self.surfaces.remove(TRASH_PATH);
        self.make_trash_surface();
        if let Some(s) = self.surfaces.get(TRASH_PATH).cloned() {
            for v in self.views.iter_mut() {
                if matches!(&v.kind, Kind::App { path } if path == TRASH_PATH) {
                    let _ = v.set_surface(&self.comp, &s);
                }
            }
        }
    }

    pub fn on_icons(&mut self) {
        while let Ok((path, bmp)) = self.worker.results.try_recv() {
            let Some(mut b) = bmp else { continue };
            if crate::prefs::get().dock_plates && b.w == b.h && !icons::is_themed(&path) {
                b.bgra = if path == TRASH_PATH || path == downloads_key() {
                    nacre_core::iconplate::free(&b.bgra, b.w)
                } else {
                    nacre_core::iconplate::plate(&b.bgra, b.w)
                };
            }
            if !icons::is_themed(&path) {
                let p = crate::prefs::get();
                b.bgra = nacre_core::iconplate::style_icon_v(
                    &b.bgra,
                    p.icon_style,
                    p.icon_tint,
                    crate::prefs::icon_dark(),
                );
            }
            match self.gfx.surface_from_bgra(b.w, b.h, &b.bgra) {
                Ok(surface) => {
                    for v in self.views.iter_mut() {
                        if matches!(&v.kind, Kind::App { path: p } if *p == path) {
                            let _ = v.set_surface(&self.comp, &surface);
                        }
                    }
                    self.surfaces.insert(path, surface);
                }
                Err(e) => log::line("icons", &format!("unggah gagal: {e}")),
            }
        }
    }

    pub(super) fn apply_prefs(&mut self) {
        let s = crate::prefs::get();
        let key = (
            s.icon_style,
            s.icon_tint,
            s.icon_pack,
            crate::prefs::icon_dark(),
        );
        if key != self.icon_key {
            self.icon_key = key;
            nacre_platform::iconpack::configure(&paths::config_dir().join("iconpack"), s.icon_pack);
            self.surfaces.clear();
            self.requested.clear();
            self.signature.clear();
            self.refresh_windows();
        }
        if s.dock_plates != self.plates {
            self.plates = s.dock_plates;
            self.surfaces.clear();
            self.requested.clear();
            self.signature.clear();
            self.refresh_windows();
        }
        if self.off == s.dock_enabled {
            self.off = !s.dock_enabled;
            if self.off {
                nacre_platform::shell::appbar::release_bottom();
            } else if self.opts.appbar {
                self.reserve_strip();
            }
            log::line(
                "dock",
                if self.off {
                    "dinonaktifkan"
                } else {
                    "diaktifkan"
                },
            );
            self.update_visibility();
        }
        let (mode, appbar) = (s.dock.mode(), s.dock.reserves());
        if mode != self.opts.mode || appbar != self.opts.appbar {
            let was_appbar = self.opts.appbar;
            self.opts.mode = mode;
            self.opts.appbar = appbar;
            if mode == Mode::Always {
                self.strip = None;
                self.edge_armed = false;
            } else if self.strip.is_none() {
                self.strip = Strip::new(self.hwnd, STRIP_ENTER, STRIP_LEAVE).ok();
                self.place_strip();
            }
            if appbar {
                self.reserve_strip();
            } else if was_appbar {
                nacre_platform::shell::appbar::release_bottom();
            }
            self.sync_strip();
            self.update_visibility();
            log::line("dock", &format!("perilaku → {mode:?} cadangan={appbar}"));
        }
        self.apply_metrics();
        let (themed, dark) = (
            crate::prefs::look(crate::prefs::Role::Dock),
            crate::prefs::look(crate::prefs::Role::Dark),
        );
        let base = GlassParams::dark((10.0, 10.0), 22.0 * self.dpi);
        let _ = self.panel.set_params(crate::prefs::apply(base, &themed));
        let _ = self
            .panel
            .set_radius(22.0 * self.dpi * crate::prefs::round(crate::prefs::Role::Dock));
        let pd = crate::prefs::apply(base, &dark);
        if let Some(t) = self.tooltip.as_mut() {
            t.apply_look(pd);
        }
        if let Some(p) = self.preview.as_mut() {
            let _ = p.panel_mut().set_params(pd);
        }
        if let Some(m) = self.menu.as_mut() {
            m.apply_look(pd);
        }
    }

    pub(super) fn reload_icon(&mut self, path: &str) {
        self.surfaces.remove(path);
        self.requested.insert(path.to_string());
        let _ = self.worker.requests.send(path.to_string());
    }
}
