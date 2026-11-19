//! Process lifecycle safety nets and the shutdown sequence.
//!
//! Anything Nexora hides or replaces on the system (taskbar, app bar, cursors, parked windows)
//! is released here on every exit path: normal exit, panic, Ctrl-C / console close, and drop.

use crate::fx;
use nexora_platform::{log, shell::taskbar};
use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, CTRL_CLOSE_EVENT, SetConsoleCtrlHandler};
use windows::Win32::System::Threading::{GetCurrentProcess, TerminateProcess};
use windows::core::BOOL;

pub fn release_all(why: &str) {
  log::line("exit", why);
  nexora_platform::shell::appbar::release();
  log::line("exit", "appbar ok");
  fx::restore_all();
  log::line("exit", "fx ok");
  taskbar::restore();
  log::line("exit", "taskbar ok");
}

unsafe extern "system" fn ctrl_handler(t: u32) -> BOOL {
  if t == CTRL_C_EVENT || t == CTRL_BREAK_EVENT || t == CTRL_CLOSE_EVENT {
    release_all("ctrl-handler");
  }
  BOOL(0)
}

/// Releases system surfaces when dropped (early return or unwinding).
pub struct ExitGuard;

impl Drop for ExitGuard {
  fn drop(&mut self) {
    release_all("drop");
  }
}

/// Installs the panic hook and console control handler, and returns the drop guard.
/// Keep the guard alive for the whole run.
pub fn install_safety_nets() -> ExitGuard {
  std::panic::set_hook(Box::new(|info| {
    log::line("panic", &info.to_string());
    release_all("panic");
  }));
  unsafe {
    let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
  }
  ExitGuard
}

/// Stops background services, releases system surfaces and ends the process.
pub fn shutdown() -> ! {
  crate::switcher::shutdown();
  crate::stage::shutdown();
  nexora_platform::clipboard::shutdown();
  crate::macshortcuts::shutdown();
  release_all("exit");
  unsafe {
    let _ = TerminateProcess(GetCurrentProcess(), 0);
  }
  std::process::exit(0);
}
