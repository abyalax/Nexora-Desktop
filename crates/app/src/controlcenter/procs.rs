use super::*;

pub(super) const KNOB: (f32, f32) = (42.0, 28.0);

pub(super) const CARD_R: f32 = 0.34;

pub(super) const DROP_IDLE: [Color; 3] = [
  Color {
    A: 45,
    R: 0,
    G: 0,
    B: 0,
  },
  Color {
    A: 250,
    R: 255,
    G: 255,
    B: 255,
  },
  Color {
    A: 0,
    R: 255,
    G: 255,
    B: 255,
  },
];

pub(super) const DROP_ACTIVE: [Color; 3] = [
  Color { A: 0, R: 0, G: 0, B: 0 },
  Color {
    A: 125,
    R: 255,
    G: 255,
    B: 255,
  },
  Color {
    A: 40,
    R: 255,
    G: 255,
    B: 255,
  },
];

pub(super) fn raindrop(comp: &Compositor, thumb: &[SpriteVisual], active: bool) {
  let (cols, from, to) = if active {
    (DROP_ACTIVE, 1.0, 1.22)
  } else {
    (DROP_IDLE, 1.22, 1.0)
  };
  for (j, v) in thumb.iter().enumerate() {
    if let Ok(b) = v.Brush() {
      let _ = motion::color(comp, &b, cols[j.min(2)], 150);
    }
    if let Ok(sz) = v.Size() {
      let _ = v.SetCenterPoint(Vector3 {
        X: sz.X / 2.0,
        Y: sz.Y / 2.0,
        Z: 0.0,
      });
    }
    let _ = motion::vec3(
      comp,
      v,
      "Scale",
      Vector3 {
        X: from,
        Y: from,
        Z: 1.0,
      },
      Vector3 { X: to, Y: to, Z: 1.0 },
      170,
    );
  }
}

pub(super) unsafe extern "system" fn ctl_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_RADIO => {
        with(|c| {
          if let Some(r) = &c.radios {
            while let Ok(s) = r.snapshots.try_recv() {
              c.wifi = s.wifi;
              c.bt = s.bluetooth;
            }
          }
          c.ssid = wifi::ssid();
          if c.panel.is_some() && !c.edit {
            c.build();
          }
        });
        LRESULT(0)
      }
      WM_MEDIA => {
        with(|c| {
          let mut changed = false;
          if let Some(m) = &c.media {
            while let Ok(s) = m.snapshots.try_recv() {
              c.track = Some((s.title, s.artist, s.has_media, s.playing));
              match s.art {
                ArtUpdate::New(b) => c.art = Some(b),
                ArtUpdate::None => c.art = None,
                ArtUpdate::Unchanged => {}
              }
              changed = true;
            }
          }
          if changed && c.panel.is_some() && !c.edit {
            c.build();
          }
        });
        LRESULT(0)
      }
      WM_VALUES => {
        with(|c| c.update_values());
        LRESULT(0)
      }
      WM_TIMER if wp.0 == T_CLOSE => {
        with(|c| c.finish_close());
        LRESULT(0)
      }
      WM_TIMER if wp.0 == T_REMOVE => {
        with(|c| c.finish_remove());
        LRESULT(0)
      }
      WM_TIMER if wp.0 == T_INTRO => {
        with(|c| c.start_intro());
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}

pub(super) fn xy(lp: LPARAM) -> (f32, f32) {
  ((lp.0 as i16) as f32, ((lp.0 >> 16) as i16) as f32)
}

pub(super) unsafe extern "system" fn panel_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
      m if m == WM_CLOSE_CC => {
        close();
        LRESULT(0)
      }
      WM_LBUTTONDOWN => {
        let (x, y) = pxy(lp);
        with(|c| {
          if c.closing {
            return;
          }
          let Some(p) = c.panel.as_ref() else { return };
          if hit(p.edit_btn, x, y) {
            if c.edit {
              c.leave_edit()
            } else {
              c.enter_edit()
            }
            return;
          }
          if let Some((_, cam)) = p.chip.filter(|(r, _)| hit(*r, x, y)) {
            c.close();
            launch::open(if cam {
              "ms-settings:privacy-webcam"
            } else {
              "ms-settings:privacy-microphone"
            });
            return;
          }
          let edit = c.edit;
          let Some(i) = p
            .mods
            .iter()
            .position(|m| hit(m.rect, x, y) || (edit && (hit(m.badge, x, y) || hit(m.grip, x, y))))
          else {
            return;
          };
          let m = &p.mods[i];
          if edit {
            if hit(m.badge, x, y) {
              let k = m.placed.kind;
              c.remove(k);
            } else if hit(m.grip, x, y) && c.pending_remove.is_none() {
              c.start_resize(i);
              SetCapture(hwnd);
            } else if c.pending_remove.is_none() {
              c.start_drag(i, x, y);
              SetCapture(hwnd);
            }
          } else if m.track.map(|t| hit(t, x, y)).unwrap_or(false) {
            if let Some(p) = c.panel.as_mut() {
              p.slider = Some(i);
              raindrop(&c.comp, &p.mods[i].thumb, true);
            }
            SetCapture(hwnd);
            c.set_slider(i, x, y, true);
          } else {
            c.click(i, x, y);
          }
        });
        LRESULT(0)
      }
      WM_MOUSEMOVE => {
        let (x, y) = pxy(lp);
        with(|c| {
          let slider = c.panel.as_ref().and_then(|p| p.slider);
          if let Some(i) = slider {
            c.set_slider(i, x, y, true);
            return;
          }
          if c.panel.as_ref().is_some_and(|p| p.resize.is_some()) {
            c.resize_move(x, y);
            return;
          }
          if c.panel.as_ref().is_some_and(|p| p.drag.is_some()) {
            c.drag_move(x, y);
            return;
          }
          let mut t = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
          };
          let _ = TrackMouseEvent(&mut t);
          let hit = c
            .panel
            .as_ref()
            .filter(|p| p.drag.is_none() && !c.edit && !c.closing)
            .and_then(|p| p.mods.iter().position(|m| hit(m.rect, x, y)));
          c.set_hover(hit);
        });
        LRESULT(0)
      }
      WM_MOUSELEAVE => {
        with(|c| c.set_hover(None));
        LRESULT(0)
      }
      m if m == MENU_PICK => {
        let id = wp.0;
        with(|c| c.pick_size(id));
        LRESULT(0)
      }
      m if m == MENU_CLOSE => {
        with(|c| {
          if let Some(mn) = &c.menu {
            mn.hide();
          }
        });
        LRESULT(0)
      }
      WM_RBUTTONUP => {
        let (x, y) = pxy(lp);
        with(|c| {
          if c.closing {
            return;
          }
          let Some(p) = c.panel.as_ref() else { return };
          let Some(i) = p.mods.iter().position(|m| hit(m.rect, x, y)) else {
            return;
          };
          if c.edit {
            c.open_size_menu(i, x, y);
          } else if matches!(p.mods[i].placed.kind, Kind::Wifi | Kind::Bluetooth) {
            c.open_picker(i);
          }
        });
        LRESULT(0)
      }
      WM_LBUTTONUP => {
        let (x, y) = pxy(lp);
        with(|c| {
          let (slider, drag) = match c.panel.as_mut() {
            Some(p) => (p.slider.take(), p.drag),
            None => (None, None),
          };
          if let (Some(i), Some(p)) = (slider, c.panel.as_ref()) {
            if let Some(m) = p.mods.get(i) {
              raindrop(&c.comp, &m.thumb, false);
            }
          }
          if slider.is_some() || drag.is_some() {
            let _ = ReleaseCapture();
          }
          if drag.is_some() {
            let _ = (x, y);
            c.end_drag();
          }
          if c.panel.as_ref().is_some_and(|p| p.resize.is_some()) {
            let _ = ReleaseCapture();
            c.end_resize();
          }
        });
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}

pub(super) unsafe extern "system" fn gallery_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  unsafe {
    match msg {
      WM_MOUSEMOVE => {
        let (x, y) = xy(lp);
        with(|c| {
          let Some(g) = c.gallery.as_mut() else { return };
          if !g.tracking {
            let mut t = TRACKMOUSEEVENT {
              cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
              dwFlags: TME_LEAVE,
              hwndTrack: hwnd,
              dwHoverTime: 0,
            };
            g.tracking = TrackMouseEvent(&mut t).is_ok();
          }
          let i = g.conts.iter().position(|(r, _, _)| hit(*r, x, y));
          c.set_gallery_hover(i);
        });
        LRESULT(0)
      }
      WM_MOUSELEAVE => {
        with(|c| {
          if let Some(g) = c.gallery.as_mut() {
            g.tracking = false;
          }
          c.set_gallery_hover(None);
        });
        LRESULT(0)
      }
      WM_LBUTTONUP => {
        let (x, y) = xy(lp);
        with(|c| {
          let Some(g) = c.gallery.as_ref() else { return };
          if hit(g.done, x, y) {
            c.leave_edit();
            return;
          }
          if let Some(cat) = g.cats.iter().find(|(_, r)| hit(*r, x, y)).map(|(c, _)| *c) {
            if let Some(g) = c.gallery.as_mut() {
              g.cat = cat;
              g.scroll = 0.0;
            }
            c.refresh_gallery();
            return;
          }
          if let Some(k) = g.cards.iter().find(|(_, r)| hit(*r, x, y)).map(|(k, _)| *k) {
            c.add(k);
          }
        });
        LRESULT(0)
      }
      WM_MOUSEWHEEL => {
        let delta = ((wp.0 >> 16) as i16) as f32;
        with(|c| {
          if let Some(g) = c.gallery.as_mut() {
            let max = (g.content_h - (g.popup.rect.3 as f32 - 80.0 * g.dpi)).max(0.0);
            g.scroll = (g.scroll - delta / 120.0 * 60.0 * g.dpi).clamp(0.0, max);
          }
          c.refresh_gallery();
        });
        LRESULT(0)
      }
      WM_CHAR => {
        let ch = char::from_u32(wp.0 as u32).unwrap_or('\0');
        if !ch.is_control() {
          with(|c| {
            if let Some(g) = c.gallery.as_mut() {
              g.query.push(ch);
              g.scroll = 0.0;
            }
            c.refresh_gallery();
          });
        }
        LRESULT(0)
      }
      WM_KEYDOWN => {
        if wp.0 as u16 == VK_BACK.0 {
          with(|c| {
            if let Some(g) = c.gallery.as_mut() {
              g.query.pop();
            }
            c.refresh_gallery();
          });
        } else if wp.0 as u16 == VK_ESCAPE.0 {
          with(|c| c.leave_edit());
        }
        LRESULT(0)
      }
      _ => DefWindowProcW(hwnd, msg, wp, lp),
    }
  }
}
