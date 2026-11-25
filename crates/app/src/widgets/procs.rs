use super::*;

pub(super) unsafe extern "system" fn ctl_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_TIMER => {
      match wp.0 {
        T_TICK => {
          with(|m| m.tick());
        }
        T_REMOVE => {
          with(|m| m.finish_remove());
        }
        T_GCLOSE => {
          with(|m| m.finish_close_gallery());
        }
        T_SNAP => {
          with(|m| m.step_snap());
        }
        _ => {}
      }
      LRESULT(0)
    }
    prefs::WM_PREFS => {
      with(|m| {
        m.rebuild_all();
        if m.gallery.is_some() {
          m.refresh_gallery();
        }
      });
      LRESULT(0)
    }
    m if m == MENU_PICK => {
      let id = wp.0;
      with(|mg| {
        if let Some(mn) = &mg.menu {
          mn.hide();
        }
        let Some(i) = mg.menu_for.take() else { return };
        match id {
          CMD_EDIT => mg.open_gallery(),
          CMD_REMOVE => mg.remove(i),
          _ => {}
        }
      });
      LRESULT(0)
    }
    m if m == MENU_CLOSE => {
      with(|mg| {
        if let Some(mn) = &mg.menu {
          mn.hide();
        }
      });
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}

pub(super) unsafe extern "system" fn widget_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    WM_WINDOWPOSCHANGING => {
      let p = lp.0 as *mut WINDOWPOS;
      if !p.is_null() && !FLOAT.load(Ordering::Relaxed) && (*p).flags.0 & SWP_NOZORDER.0 == 0 {
        (*p).hwndInsertAfter = HWND_BOTTOM;
      }
      DefWindowProcW(hwnd, msg, wp, lp)
    }
    WM_LBUTTONDOWN => {
      let (x, y) = xy(lp);
      with(|m| {
        m.finish_snap();
        let Some(i) = m.index_of(hwnd) else { return };
        if m.badge_hit(i, x, y) {
          m.remove(i);
          return;
        }
        let c = cursor();
        let pos = m.list[i].pos;
        m.drag = Some(Drag {
          idx: i,
          grab: (c.0 - pos.0, c.1 - pos.1),
          start: c,
          moved: false,
        });
        SetCapture(hwnd);
      });
      LRESULT(0)
    }
    WM_MOUSEMOVE => {
      with(|m| {
        let Some(i) = m.index_of(hwnd) else { return };
        if let Some(dg) = m.drag.as_ref().filter(|dg| dg.idx == i) {
          let c = cursor();
          let (grab, start, moved) = (dg.grab, dg.start, dg.moved);
          if !moved && ((c.0 - start.0).abs() > 4 || (c.1 - start.1).abs() > 4) {
            if let Some(dg) = m.drag.as_mut() {
              dg.moved = true;
            }
            m.lift(i);
          }
          if m.drag.as_ref().is_some_and(|dg| dg.moved) {
            m.drag_to(i, (c.0 - grab.0, c.1 - grab.1));
          }
          return;
        }
        if !m.list[i].tracking {
          let mut t = TRACKMOUSEEVENT {
            cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
            dwFlags: TME_LEAVE,
            hwndTrack: hwnd,
            dwHoverTime: 0,
          };
          m.list[i].tracking = TrackMouseEvent(&mut t).is_ok();
        }
        m.set_hover(i, true);
      });
      LRESULT(0)
    }
    WM_MOUSELEAVE => {
      with(|m| {
        if let Some(i) = m.index_of(hwnd) {
          m.list[i].tracking = false;
          m.set_hover(i, false);
        }
      });
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let _ = ReleaseCapture();
      with(|m| {
        let Some(dg) = m.drag.take() else { return };
        if dg.moved {
          m.drop_at(dg.idx);
        } else if !m.edit {
          if let Some(w) = m.list.get(dg.idx) {
            Mgr::open_app(w.kind);
          }
        }
      });
      LRESULT(0)
    }
    WM_RBUTTONUP => {
      with(|m| {
        if let Some(i) = m.index_of(hwnd) {
          m.show_menu(i);
        }
      });
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}

pub(super) unsafe extern "system" fn slot_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
    WM_MOUSEACTIVATE => LRESULT(MA_NOACTIVATE as isize),
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}

pub(super) unsafe extern "system" fn gallery_proc(hwnd: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
  match msg {
    WM_LBUTTONDOWN => {
      let (x, y) = xy(lp);
      with(|m| {
        let Some(g) = m.gallery.as_ref() else { return };
        if hit(g.done, x, y) {
          m.close_gallery();
          return;
        }
        if let Some(c) = g.cats.iter().find(|(_, r)| hit(*r, x, y)).map(|(c, _)| *c) {
          if let Some(g) = m.gallery.as_mut() {
            g.cat = c;
          }
          m.refresh_gallery();
          return;
        }
        if let Some(k) = g.cards.iter().find(|(_, r)| hit(*r, x, y)).map(|(k, _)| *k) {
          if let Some(g) = m.gallery.as_mut() {
            g.press = Some((k, cursor()));
          }
          SetCapture(hwnd);
        }
      });
      LRESULT(0)
    }
    WM_MOUSEMOVE => {
      let (x, y) = xy(lp);
      with(|m| {
        let c = cursor();
        if let Some(dg) = m.drag.as_ref() {
          let (i, grab) = (dg.idx, dg.grab);
          m.drag_to(i, (c.0 - grab.0, c.1 - grab.1));
          return;
        }
        let press = m.gallery.as_ref().and_then(|g| g.press);
        if let Some((k, start)) = press {
          if (c.0 - start.0).abs() > 6 || (c.1 - start.1).abs() > 6 {
            let (w, h) = widgets::size_of(k, m.dpi);
            let grab = ((w / 2.0) as i32, (h / 2.0) as i32);
            FLOAT.store(true, Ordering::Relaxed);
            if let Some(i) = m.add(k, Some((c.0 - grab.0, c.1 - grab.1))) {
              m.lift(i);
              m.drag = Some(Drag {
                idx: i,
                grab,
                start,
                moved: true,
              });
            }
            if let Some(g) = m.gallery.as_mut() {
              g.press = None;
            }
          }
          return;
        }
        let Some(g) = m.gallery.as_mut() else { return };
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
        m.gallery_hover(i);
      });
      LRESULT(0)
    }
    WM_MOUSELEAVE => {
      with(|m| {
        if let Some(g) = m.gallery.as_mut() {
          g.tracking = false;
        }
        m.gallery_hover(None);
      });
      LRESULT(0)
    }
    WM_LBUTTONUP => {
      let _ = ReleaseCapture();
      with(|m| {
        if let Some(dg) = m.drag.take() {
          m.drop_at(dg.idx);
          return;
        }
        let press = m.gallery.as_mut().and_then(|g| g.press.take());
        if let Some((k, _)) = press {
          m.add(k, None);
        }
      });
      LRESULT(0)
    }
    WM_CHAR => {
      let ch = char::from_u32(wp.0 as u32).unwrap_or('\0');
      if !ch.is_control() {
        with(|m| {
          if let Some(g) = m.gallery.as_mut() {
            g.query.push(ch);
          }
          m.refresh_gallery();
        });
      }
      LRESULT(0)
    }
    WM_KEYDOWN => {
      if wp.0 as u16 == VK_BACK.0 {
        with(|m| {
          if let Some(g) = m.gallery.as_mut() {
            g.query.pop();
          }
          m.refresh_gallery();
        });
      } else if wp.0 as u16 == VK_ESCAPE.0 {
        with(|m| m.close_gallery());
      }
      LRESULT(0)
    }
    WM_CLOSE => {
      with(|m| m.close_gallery());
      LRESULT(0)
    }
    _ => DefWindowProcW(hwnd, msg, wp, lp),
  }
}
