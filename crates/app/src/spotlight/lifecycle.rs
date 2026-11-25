use super::*;

pub fn init(comp: &Compositor) -> windows_core::Result<()> {
  unsafe {
    let hi: HINSTANCE = GetModuleHandleW(None)?.into();
    let class = w!("NexoraSpotCtl");
    RegisterClassW(&WNDCLASSW {
      lpfnWndProc: Some(ctl_proc),
      hInstance: hi,
      lpszClassName: class,
      ..Default::default()
    });
    let hwnd = CreateWindowExW(
      WINDOW_EX_STYLE(0),
      class,
      w!("nexora-spot-ctl"),
      WS_POPUP,
      0,
      0,
      0,
      0,
      Some(HWND_MESSAGE),
      None,
      Some(hi),
      None,
    )?;
    if RegisterHotKey(Some(hwnd), HOTKEY_ID, MOD_CONTROL | MOD_NOREPEAT, VK_SPACE.0 as u32).is_err() {
      log::line(
        "spotlight",
        "Ctrl+Space sudah dipakai program lain; hot key tidak aktif",
      );
    }
    if RegisterHotKey(
      Some(hwnd),
      HOTKEY_APPS,
      MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT,
      VK_SPACE.0 as u32,
    )
    .is_err()
    {
      log::line(
        "spotlight",
        "Ctrl+Shift+Space dipakai program lain; hot key Apps tidak aktif",
      );
    }
    let h = hwnd.0 as isize;
    CTL.with(|c| {
      *c.borrow_mut() = Some(Ctl {
        hwnd,
        comp: comp.clone(),
      })
    });
    prefs::listen(hwnd);
    clipboard::set_enabled(prefs::get().clipboard_history);
    apps::refresh(move || {
      let _ = PostMessageW(Some(HWND(h as *mut _)), WM_APPS, WPARAM(0), LPARAM(0));
    });
  }
  Ok(())
}

#[allow(dead_code)]
pub fn shutdown() {
  CTL.with(|c| {
    if let Some(ctl) = c.borrow().as_ref() {
      unsafe {
        let _ = UnregisterHotKey(Some(ctl.hwnd), HOTKEY_ID);
        let _ = UnregisterHotKey(Some(ctl.hwnd), HOTKEY_APPS);
      }
    }
  });
}

pub fn toggle() {
  toggle_mode(Mode::Search)
}

pub fn toggle_mode(mode: Mode) {
  let open_mode = SPOT.with(|c| c.borrow().as_ref().map(|s| s.mode));
  match open_mode {
    Some(m) if m == mode => {
      close();
      return;
    }
    Some(_) => {
      with_spot(|s| s.set_mode(mode));
      return;
    }
    None => {}
  }
  let comp = CTL.with(|c| c.borrow().as_ref().map(|c| c.comp.clone()));
  let Some(comp) = comp else { return };
  match create(&comp, mode) {
    Ok(s) => {
      SPOT.with(|c| *c.borrow_mut() = Some(s));
      with_spot(|s| {
        s.reset_state(mode);
        s.render();
        let root = s.popup.root().clone();
        let _ = root.SetOpacity(0.0);
        s.popup.show_active();
        let _ = crate::motion::scalar(&s.comp, &root, "Opacity", 0.0, 1.0, 80);
        let _ = root.SetOpacity(1.0);
      });
      let h = CTL
        .with(|c| c.borrow().as_ref().map(|c| c.hwnd.0 as isize))
        .unwrap_or(0);
      apps::refresh(move || unsafe {
        let _ = PostMessageW(Some(HWND(h as *mut _)), WM_APPS, WPARAM(0), LPARAM(0));
      });
    }
    Err(e) => log::line("spotlight", &format!("gagal: {e}")),
  }
}

pub fn close() {
  let s = SPOT.with(|c| c.borrow_mut().take());
  if let Some(s) = s {
    finish_closing();
    let root = s.popup.root().clone();
    let _ = crate::motion::scalar(&s.comp, &root, "Opacity", 1.0, 0.0, 90);
    let _ = root.SetOpacity(0.0);
    CLOSING.with(|c| *c.borrow_mut() = Some(s));
    let ctl = CTL.with(|c| c.borrow().as_ref().map(|c| c.hwnd));
    if let Some(h) = ctl {
      unsafe {
        SetTimer(Some(h), T_FINISH, 110, None);
      }
    }
  }
}

pub(super) fn finish_closing() {
  let s = CLOSING.with(|c| c.borrow_mut().take());
  if let Some(s) = s {
    let h = s.popup.hwnd;
    drop(s);
    unsafe {
      let _ = DestroyWindow(h);
    }
    nexora_platform::sys::trim_later(5000);
  }
}

pub(super) fn create(comp: &Compositor, mode: Mode) -> windows_core::Result<Box<Spot>> {
  let screen = sys::screen_size();
  let glass = prefs::glass((10.0, 10.0), 27.0, prefs::Role::Dock);
  let mut popup = Popup::new_with(
    comp,
    w!("NexoraSpotlight"),
    Some(spot_proc),
    WINDOW_EX_STYLE(0),
    glass,
    true,
  )?;
  let dpi = unsafe { GetDpiForWindow(popup.hwnd) }.max(96) as f32 / 96.0;
  let tier = {
    let c = sys::conditions();
    if !c.transparency || c.battery_saver || c.remote_session {
      nexora_ui::glass::Tier::Solid
    } else {
      nexora_ui::glass::Tier::Full
    }
  };
  let _ = popup.panel.set_tier(tier);
  let g = geo(dpi, screen);
  popup.set_rect(g.win.0, g.win.1, g.win.2, g.win.3)?;
  let idle: Rect = ((g.gw - g.ws) / 2.0, 0.0, g.ws, g.h);
  popup.panel.set_rect(idle.0, idle.1, idle.2, idle.3)?;
  popup.panel.set_radius(g.h / 2.0)?;
  popup.content.SetSize(Vector2 {
    X: g.win.2 as f32,
    Y: g.win.3 as f32,
  })?;
  let clip = comp.CreateInsetClip()?;
  popup.content.SetClip(&clip)?;
  let side = comp.CreateContainerVisual()?;
  popup.root().Children()?.InsertAtTop(&side)?;
  let gfx = Gfx::shared(comp)?;
  let apps = apps::snapshot();
  let names = apps.iter().map(|a| a.name.clone()).collect();
  let quick = model::parse_quick_keys(&std::fs::read_to_string(quick_file()).unwrap_or_default());
  let _ = mode;
  let s = Box::new(Spot {
    popup,
    gfx,
    comp: comp.clone(),
    dpi,
    screen,
    mode: Mode::Search,
    query: String::new(),
    rows: vec![],
    sel: 0,
    scroll: 0,
    apps,
    names,
    cat: None,
    filter: None,
    files: vec![],
    files_gen: 0,
    shared_files: Arc::new(Mutex::new((0, vec![]))),
    quick,
    editing: None,
    edit_buf: String::new(),
    hits: vec![],
    circles: vec![],
    text_cache: HashMap::new(),
    icon_cache: HashMap::new(),
    geo: g,
    cur: idle,
    buttons: false,
    buttons_intro: false,
    side_fading: false,
    hover_mode: None,
    popped: None,
    tracking: false,
    clip,
    side,
    sections: vec![],
    expanded: vec![],
    pscroll: 0.0,
    content_h: 0.0,
    view: (0.0, 0.0),
    cell_y: vec![],
    more_open: false,
    headers: vec![],
    recents: Arc::new(Mutex::new(vec![])),
    thumbs: Arc::new(Mutex::new(HashMap::new())),
    thumb_surf: HashMap::new(),
    recents_requested: false,
    morphing: false,
    ftype: None,
  });
  s.set_region(idle);
  Ok(s)
}
