//! Window creation, geometry, region and activation-strip placement.

use super::*;

impl Dock {
  pub fn create(host: &Host, main_hwnd: HWND, opts: Options) -> windows_core::Result<HWND> {
    unsafe {
      let hi: HINSTANCE = GetModuleHandleW(None)?.into();
      let class = w!("NacreDock");
      RegisterClassW(&WNDCLASSW {
        lpfnWndProc: Some(proc),
        hInstance: hi,
        lpszClassName: class,
        hCursor: LoadCursorW(None, IDC_ARROW)?,
        ..Default::default()
      });
      let hwnd = CreateWindowExW(
        WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
        class,
        w!("nacre-dock"),
        WS_POPUP,
        0,
        0,
        400,
        120,
        None,
        None,
        Some(hi),
        None,
      )?;
      let dpi = GetDpiForWindow(hwnd).max(96) as f32 / 96.0 * crate::prefs::ui_scale();
      let screen = sys::screen_size();
      let m = metrics_from_prefs(dpi);

      let surface = host.attach(hwnd, (400.0, 120.0))?;
      let stage = host.compositor.CreateContainerVisual()?;
      surface.root.Children()?.InsertAtTop(&stage)?;
      let panel = GlassPanel::new(
        &host.compositor,
        &stage,
        crate::prefs::glass(
          (10.0, 10.0),
          22.0 * dpi * crate::prefs::round(crate::prefs::Role::Dock),
          crate::prefs::Role::Dock,
        ),
      )?;
      let tier = opts.glass.unwrap_or_else(|| auto_tier(&sys::conditions()));
      panel.set_tier(tier)?;
      let content = host.compositor.CreateContainerVisual()?;
      stage.Children()?.InsertAtTop(&content)?;

      let gfx = Gfx::shared(&host.compositor)?;
      let hw = hwnd.0 as isize;
      let px = ((m.icon * m.max_scale * 1.15) as i32).clamp(64, 256);
      let worker = icons::spawn_worker(px, move || {
        let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_ICONS, WPARAM(0), LPARAM(0));
      });

      custom_icons::install();
      nexora_platform::iconpack::configure(&paths::config_dir().join("iconpack"), crate::prefs::get().icon_pack);
      let pins_path = paths::config_dir().join("pins.txt");
      let pins = pins::load(&pins_path);
      let recents_path = paths::config_dir().join("recents.txt");
      let recents: Vec<Pin> = pins::load(&recents_path)
        .into_iter()
        .filter(|r| r.path.to_ascii_lowercase().starts_with("shell:") || std::path::Path::new(&r.path).exists())
        .take(recents::KEEP)
        .collect();
      let mode = opts.mode;

      let mut dock = Box::new(Dock {
        hwnd,
        main_hwnd,
        comp: host.compositor.clone(),
        gfx,
        _surface: surface,
        stage,
        stage_anim: spring(&host.compositor, 0.78, 150)?,
        panel,
        content,
        props: {
          let p = host.compositor.CreatePropertySet()?;
          p.InsertScalar(&HSTRING::from("cx"), -10000.0)?;
          p.InsertScalar(&HSTRING::from("k"), 0.0)?;
          p.InsertScalar(&HSTRING::from("scroll"), 0.0)?;
          p
        },
        k_anim: spring(&host.compositor, 0.72, 85)?,
        exprs_ok: false,
        k_on: false,
        dpi,
        m,
        screen,
        win: (400, 120),
        content_w: 400.0,
        win_pos: (0, 0),
        card_y: 0.0,
        card_h: 0.0,
        margin: 8.0 * dpi,
        icon_px: px,
        icon_y: 0.0,
        dot_y: 0.0,
        pad_side: 10.0 * dpi,
        scroll_x: 0.0,
        opts,
        state: DockState::new(crate::prefs::get().dock_enabled),
        pins,
        pins_path,
        recents,
        recents_path,
        stack_files: Vec::new(),
        items: vec![],
        signature: vec![],
        icon_key: {
          let p = crate::prefs::get();
          (p.icon_style, p.icon_tint, p.icon_pack, crate::prefs::icon_dark())
        },
        seen: vec![],
        entries: vec![],
        slots: vec![],
        views: vec![],
        layout: Layout {
          cells: vec![],
          total: 0.0,
        },
        surfaces: HashMap::new(),
        requested: HashSet::new(),
        trash_full: nexora_platform::recycle::is_full(),
        worker,
        plates: crate::prefs::get().dock_plates,
        labels: HashMap::new(),
        region_full: false,
        cursor: None,
        last_apply: Instant::now(),
        last_card: (0.0, 0.0, 0.0, 0.0),
        start_open_at_press: false,
        hover: None,
        press: None,
        dragging: None,
        overlap: Overlap::default(),
        tier,
        strip: if mode == Mode::Always {
          None
        } else {
          Strip::new(hwnd, STRIP_ENTER, STRIP_LEAVE).ok()
        },
        tooltip: None,
        preview: None,
        menu: None,
        pending_menu: None,
      });
      dock.compute_geometry();
      dock.refresh_windows();
      let _ = dock.stage.SetOffset(Vector3 {
        X: 0.0,
        Y: dock.win.1 as f32 + 8.0,
        Z: 0.0,
      });
      let appbar = dock.opts.appbar;
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(dock) as isize);
      DOCK_HWND.store(hwnd.0 as isize, Ordering::Relaxed);
      windows::Win32::UI::Shell::DragAcceptFiles(hwnd, true);
      crate::prefs::listen(hwnd);
      tracker::install(on_windows_changed);
      let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
      with_dock(hwnd, |d| {
        d.animate_stage(0.0);
        d.sync_strip();
        if appbar {
          d.reserve_strip();
        }
      });
      timer(hwnd, TIMER_START, 1600);
      timer(hwnd, TIMER_TRASH, TRASH_POLL_MS);
      log::line("dock", &format!("dibuat tier={tier:?} mode={mode:?}"));
      Ok(hwnd)
    }
  }

  pub(super) fn compute_geometry(&mut self) {
    let f = geometry::frame(self.dpi, &self.m, crate::prefs::get().dock_margin as f32);
    self.margin = f.margin;
    self.card_h = f.card_h;
    self.card_y = f.card_y;
    self.icon_y = f.icon_y;
    self.dot_y = f.dot_y;
    self.win.1 = f.win_h as i32;
  }

  pub(super) fn max_content_width(&self) -> f32 {
    geometry::max_content_width(&self.slots, &self.m)
  }

  pub(super) fn content_width(&self) -> f32 {
    self.content_w.max(self.win.0 as f32)
  }

  pub(super) fn scroll_limit(&self) -> f32 {
    (self.content_width() - self.win.0 as f32).max(0.0)
  }

  pub(super) fn layout_at(&self, cursor_x: Option<f32>) -> Layout {
    layout_scrolled(
      &self.slots,
      cursor_x,
      self.content_width(),
      self.win.0 as f32,
      self.scroll_x,
      &self.m,
    )
  }

  pub(super) fn resize_window(&mut self) {
    let max_content = self.max_content_width();
    let need = max_content + 2.0 * self.pad_side + 40.0 * self.dpi;
    let w = (need.ceil() as i32).clamp(200, self.screen.0);
    self.win.0 = w;
    self.content_w = (max_content + 2.0 * self.pad_side).max(w as f32);
    self.scroll_x = self.scroll_x.clamp(0.0, self.scroll_limit());
    let _ = self.props.InsertScalar(&HSTRING::from("scroll"), self.scroll_x);
    self.win_pos = ((self.screen.0 - w) / 2, self.screen.1 - self.win.1);
    unsafe {
      let _ = SetWindowPos(
        self.hwnd,
        None,
        self.win_pos.0,
        self.win_pos.1,
        w,
        self.win.1,
        SWP_NOACTIVATE | SWP_NOZORDER,
      );
    }
    let _ = self._surface.root.SetSize(windows_numerics::Vector2 {
      X: w as f32,
      Y: self.win.1 as f32,
    });
    self.place_strip();
    log::line(
      "dock",
      &format!(
        "jendela=({},{})-({},{}) layar={}x{} dpi={}",
        self.win_pos.0,
        self.win_pos.1,
        self.win_pos.0 + w,
        self.win_pos.1 + self.win.1,
        self.screen.0,
        self.screen.1,
        self.dpi
      ),
    );
  }

  pub(super) fn on_reconfigure(&mut self) {
    let screen = sys::screen_size();
    let dpi = unsafe { GetDpiForWindow(self.hwnd) }.max(96) as f32 / 96.0 * crate::prefs::ui_scale();
    if screen == self.screen && (dpi - self.dpi).abs() < 0.01 {
      return;
    }
    log::line(
      "dock",
      &format!(
        "reconfigurasi {}x{}@{} → {}x{}@{}",
        self.screen.0, self.screen.1, self.dpi, screen.0, screen.1, dpi
      ),
    );
    self.hide_popups();
    self.tooltip = None;
    self.preview = None;
    self.screen = screen;
    self.dpi = dpi;
    self.m = metrics_from_prefs(dpi);
    self.pad_side = 10.0 * dpi;
    self.compute_geometry();
    let _ = self
      .panel
      .set_radius(22.0 * dpi * crate::prefs::round(crate::prefs::Role::Dock));
    self.last_card = (0.0, 0.0, 0.0, 0.0);
    self.signature.clear();
    self.refresh_windows();
    if self.opts.appbar {
      self.reserve_strip();
    }
  }

  pub(super) fn apply_metrics(&mut self) {
    let s = crate::prefs::get();
    let m = metrics_from_prefs(self.dpi);
    let same = (m.icon - self.m.icon).abs() < 0.01
      && (m.gap - self.m.gap).abs() < 0.01
      && (m.max_scale - self.m.max_scale).abs() < 0.001
      && (s.dock_margin as f32 * self.dpi - self.margin).abs() < 0.01;
    if same {
      return;
    }
    self.hide_popups();
    self.m = m;
    let px = ((m.icon * m.max_scale * 1.15) as i32).clamp(64, 256);
    if px != self.icon_px {
      self.icon_px = px;
      let hw = self.hwnd.0 as isize;
      self.worker = icons::spawn_worker(px, move || unsafe {
        let _ = PostMessageW(Some(HWND(hw as *mut _)), WM_ICONS, WPARAM(0), LPARAM(0));
      });
      self.surfaces.clear();
      self.requested.clear();
    }
    self.compute_geometry();
    self.last_card = (0.0, 0.0, 0.0, 0.0);
    self.signature.clear();
    self.refresh_windows();
    if self.opts.appbar {
      self.reserve_strip();
    }
    log::line(
      "dock",
      &format!(
        "metrik: ikon={} jarak={} zoom={} margin={}",
        s.dock_size, s.dock_gap, s.dock_zoom, s.dock_margin
      ),
    );
  }

  pub(super) fn reserve_strip(&mut self) {
    if self.state.off {
      return;
    }
    let d = self.dpi;
    let strip = (self.margin + self.card_h + 4.0 * d).ceil() as i32;
    let r = nexora_platform::shell::appbar::reserve_bottom(self.hwnd, self.screen.0, self.screen.1, strip);
    log::line("appbar", &format!("strip={strip} hasil={r:?}"));
  }

  pub(super) fn set_region(&mut self, full: bool) {
    self.region_full = full;
    let (l, t, r, b) = if full || self.layout.cells.is_empty() {
      (0, 0, self.win.0, self.win.1)
    } else {
      let rest = self.layout_at(None);
      let cw = rest.total + 2.0 * self.pad_side;
      let cx = (self.content_width() - cw) / 2.0 - self.scroll_x;
      (
        (cx as i32).max(0),
        self.card_y as i32,
        (cx + cw).ceil().min(self.win.0 as f32) as i32,
        (self.card_y + self.card_h).ceil() as i32,
      )
    };
    unsafe {
      let rgn = CreateRectRgn(l, t, r, b);
      SetWindowRgn(self.hwnd, Some(rgn), true);
    }
  }

  pub(super) fn card_screen_rect(&self) -> (i32, i32, i32, i32) {
    let rest = self.layout_at(None);
    let cw = rest.total + 2.0 * self.pad_side;
    let cx = (self.content_width() - cw) / 2.0 - self.scroll_x;
    let l = self.win_pos.0 + (cx as i32).max(0);
    let t = self.win_pos.1 + self.card_y as i32;
    let r = self.win_pos.0 + (cx + cw).ceil().min(self.win.0 as f32) as i32;
    (l, t, r, t + (self.card_h + self.margin) as i32)
  }

  pub(super) fn place_strip(&self) {
    if let Some(s) = &self.strip {
      s.place(self.win_pos.0, self.screen.1 - 2, self.win.0, 2);
    }
  }

  pub(super) fn sync_strip(&self) {
    if self.state.off {
      if let Some(s) = &self.strip {
        s.set_visible(false);
      }
      return;
    }
    if let Some(s) = &self.strip {
      s.set_visible(!self.overlap.fullscreen);
    }
  }

  pub(super) fn animate_stage(&self, y: f32) {
    if let Ok(v) = f32_ref(y) {
      if self.stage_anim.SetFinalValue(&v).is_ok() {
        let _ = self.stage.StartAnimation(&HSTRING::from("Offset.Y"), &self.stage_anim);
      }
    }
  }

  pub(super) fn refresh_tier(&mut self) {
    let t = self.opts.glass.unwrap_or_else(|| auto_tier(&sys::conditions()));
    if t != self.tier {
      self.tier = t;
      let _ = self.panel.set_tier(t);
      if let Some(tt) = self.tooltip.as_mut() {
        tt.set_tier(t);
      }
      if let Some(p) = self.preview.as_mut() {
        let _ = p.panel_mut().set_tier(t);
      }
      if let Some(m) = self.menu.as_mut() {
        m.set_tier(t);
      }
      log::line("dock", &format!("tier={t:?}"));
    }
  }
}
