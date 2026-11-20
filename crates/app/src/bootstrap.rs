//! Composition root: builds the host, starts each feature in a fixed order and returns the
//! running [`Application`]. A feature that fails to start is logged and skipped; the rest of the
//! application keeps running (see `docs/feature.md`, cross-cutting rule 3).

use crate::application::Application;
use crate::startup::{args::Args, demo, recovery, single_instance};
use crate::{
  controlcenter, dock, dynamic_island, fx, hotcorners, hud, launchpad, lifecycle, macshortcuts, mainwin, menubar,
  mission, paths, prefs, services, settings_win, spotlight, stage, switcher, topmost, widgets,
};
use nexora_core::settings::Settings;
use nexora_platform::log;
use nexora_ui::host::Host;
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};

/// Entry used by `main`: initialize everything, then hand control to the message loop.
pub fn run() {
  if let Some(app) = initialize() {
    app.run();
  }
}

/// Logs a failed feature start under `tag` and carries on.
fn report<E: std::fmt::Display>(tag: &str, result: Result<(), E>) {
  if let Err(e) = result {
    log::line(tag, &format!("gagal: {e}"));
  }
}

fn initialize() -> Option<Application> {
  let args = Args::parse();
  unsafe {
    let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
  }
  log::init(paths::log());

  let (instance, wake) = match single_instance::acquire() {
    single_instance::Role::Secondary => return None,
    single_instance::Role::Primary(m, e) => (m, e),
  };
  log::line(
    "start",
    &format!("hide={} demo={}", args.hide_taskbar, args.demo_surface),
  );

  recovery::restore_taskbar();
  let exit_guard = lifecycle::install_safety_nets();
  unsafe {
    let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
  }

  prefs::init();
  let pref = prefs::get();
  let host = match Host::new() {
    Ok(h) => h,
    Err(e) => {
      log::line("host", &format!("gagal: {e}"));
      return None;
    }
  };

  let main_hwnd = mainwin::create().ok()?;
  single_instance::listen(wake, main_hwnd);
  mainwin::add_tray(main_hwnd);
  if let Some(s) = args.exit_after {
    mainwin::set_exit_timer(main_hwnd, s);
  }

  start_dock(&host, main_hwnd, &args, &pref);
  start_island(&host, &args, &pref);
  start_desktop_features(&host, &args);

  let hud = if args.no_hud {
    None
  } else {
    hud::Hud::create(&host, args.glass).ok()
  };
  let demo = if args.demo_surface {
    demo::create(&host).ok()
  } else {
    None
  };

  apply_taskbar_policy(main_hwnd, &args, &pref);

  Some(Application {
    _exit: exit_guard,
    _instance: instance,
    _hud: hud,
    _demo: demo,
    _host: host,
    _main_hwnd: main_hwnd,
  })
}

fn start_dock(host: &Host, main_hwnd: HWND, args: &Args, pref: &Settings) {
  if args.no_dock {
    return;
  }
  let created = dock::Dock::create(
    host,
    main_hwnd,
    dock::Options {
      appbar: args.appbar || (args.dock_mode.is_none() && pref.dock.reserves()),
      mode: args.dock_mode.unwrap_or(pref.dock.mode()),
      glass: args.glass,
    },
  );
  match created {
    Ok(h) => dock::cli_commands(h, args.add_app.clone(), args.set_icon.clone()),
    Err(e) => log::line("dock", &format!("gagal dibuat: {e}")),
  }
}

fn start_island(host: &Host, args: &Args, pref: &Settings) {
  if args.no_island {
    return;
  }
  let created = dynamic_island::Notch::create(
    host,
    dynamic_island::Options {
      glass: args.glass,
      mode: args.island_mode.unwrap_or(pref.notch.mode()),
      reserve: args.island_mode.is_none() && pref.notch.reserves(),
    },
  );
  if let Err(e) = created {
    log::line("notch", &format!("gagal dibuat: {e}"));
  }
}

/// Everything besides Dock and Dynamic Island. Order matters: several of these hook into the
/// window tracker and the shared compositor that the two features above have just created.
fn start_desktop_features(host: &Host, args: &Args) {
  if args.open_settings {
    settings_win::open(&host.compositor);
  }
  report("mission", mission::init(&host.compositor));
  report("launchpad", launchpad::init(&host.compositor));
  recovery::restore_cursors();
  report("macshortcuts", macshortcuts::init());
  report("clipboard", nexora_platform::clipboard::init());
  report("spotlight", spotlight::init(&host.compositor));
  report("stage", stage::init(&host.compositor));
  report("controlcenter", controlcenter::init(&host.compositor));
  report("switcher", switcher::init(&host.compositor));
  report("menubar", menubar::init(&host.compositor));
  report("topmost", topmost::init());
  report("hotcorner", hotcorners::init());
  report("fx", fx::init(host));
  report("widgets", widgets::init(&host.compositor));
  nexora_platform::sys::trim_later(20_000);
  if args.open_widgets {
    widgets::open_gallery();
  }
  services::init();
}

fn apply_taskbar_policy(main_hwnd: HWND, args: &Args, pref: &Settings) {
  mainwin::TAKEOVER_ALLOWED.store(
    !args.no_dock && !args.keep_taskbar,
    std::sync::atomic::Ordering::Relaxed,
  );
  prefs::listen(main_hwnd);
  if pref.autostart {
    mainwin::apply_autostart();
  }
  if args.hide_taskbar || (pref.takeover && pref.dock_enabled && !args.no_dock && !args.keep_taskbar) {
    mainwin::set_taskbar_hidden(true);
  }
}
