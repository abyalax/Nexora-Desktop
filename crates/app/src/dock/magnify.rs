//! Magnifikasi ikon (ekspresi komposisi dan jalur animasi per ikon).

use super::*;

impl Dock {
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
}
