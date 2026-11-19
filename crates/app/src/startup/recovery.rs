//! Undo what a previous run that died abnormally left behind (hidden taskbar, replaced cursors).
//! `nexora-guard` covers the same ground from outside the process; this is the in-process pass.

use crate::paths;
use nexora_platform::{cursors, log, shell::taskbar};

pub fn restore_taskbar() {
  taskbar::recover_leftover(&paths::marker());
}

pub fn restore_cursors() {
  if cursors::recover_leftover(&paths::config_dir().join("cursors_replaced.flag")) {
    log::line("cursor", "kursor sistem dipulihkan");
  }
}
