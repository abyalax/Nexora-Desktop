//! Finder actions that need a guard: the shut-down confirmation (F-9) and the corner close (F-10).

use crate::shared::geometry::Rect;

/// A power command of the logo menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerAction {
  Sleep,
  Lock,
  Restart,
  ShutDown,
}

/// Whether `action` must be confirmed first (FLB-03). Decision: only the two commands that end the session and can
/// lose unsaved work, Restart and Shut Down. Sleep and Lock are reversible and stay one click.
pub fn needs_confirm(action: PowerAction, confirmation_on: bool) -> bool {
  confirmation_on && matches!(action, PowerAction::Restart | PowerAction::ShutDown)
}

/// Keys the confirmation dialog reacts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogKey {
  Enter,
  Escape,
  Other,
}

/// What the dialog does with a key: Enter confirms, Esc cancels, everything else is ignored.
pub fn dialog_result(key: DialogKey) -> Option<bool> {
  match key {
    DialogKey::Enter => Some(true),
    DialogKey::Escape => Some(false),
    DialogKey::Other => None,
  }
}

/// Hit-test rectangle of the corner-close target (FLB-05): a `size` square in the top-right corner of the bar's screen.
pub fn corner_rect(screen: Rect, size: f32) -> Rect {
  let size = size.max(0.0).min(screen.w).min(screen.h);
  Rect::new(screen.right() - size, screen.y, size, size)
}

/// Whether the pointer is on the corner target.
pub fn in_corner(screen: Rect, size: f32, px: f32, py: f32) -> bool {
  corner_rect(screen, size).contains(px, py)
}

/// The foreground window as the platform describes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Target<'a> {
  /// Window class name.
  pub class: &'a str,
  /// Executable file name, for example `explorer.exe`.
  pub exe: &'a str,
  pub pid: u32,
  pub maximized: bool,
}

/// Window classes of the desktop and the shell that must never be closed from the corner.
const SHELL_CLASSES: [&str; 5] = [
  "Progman",
  "WorkerW",
  "Shell_TrayWnd",
  "Shell_SecondaryTrayWnd",
  "NotifyIconOverflowWindow",
];

/// Whether the corner may close this window (FLB-05): it must be maximized and must not be the desktop, the shell or
/// Nexora itself (`own_pid`). The check is a deny list on purpose, so an unknown app is closable like any other.
pub fn can_close(t: &Target, own_pid: u32) -> bool {
  t.maximized
    && t.pid != own_pid
    && !SHELL_CLASSES.iter().any(|c| c.eq_ignore_ascii_case(t.class))
    && !t.exe.to_ascii_lowercase().starts_with("nexora")
}

#[cfg(test)]
mod tests {
  use super::*;

  const SCREEN: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1920.0,
    h: 1080.0,
  };

  fn app(class: &'static str, exe: &'static str) -> Target<'static> {
    Target {
      class,
      exe,
      pid: 4242,
      maximized: true,
    }
  }

  #[test]
  fn only_restart_and_shut_down_are_confirmed() {
    for a in [PowerAction::Restart, PowerAction::ShutDown] {
      assert!(needs_confirm(a, true));
      assert!(!needs_confirm(a, false));
    }
    for a in [PowerAction::Sleep, PowerAction::Lock] {
      assert!(!needs_confirm(a, true));
    }
  }

  #[test]
  fn enter_confirms_escape_cancels_other_keys_do_nothing() {
    assert_eq!(dialog_result(DialogKey::Enter), Some(true));
    assert_eq!(dialog_result(DialogKey::Escape), Some(false));
    assert_eq!(dialog_result(DialogKey::Other), None);
  }

  #[test]
  fn corner_is_a_square_at_the_top_right() {
    let r = corner_rect(SCREEN, 26.0);
    assert_eq!((r.x, r.y, r.w, r.h), (1894.0, 0.0, 26.0, 26.0));
    assert!(in_corner(SCREEN, 26.0, 1919.0, 0.0));
    assert!(!in_corner(SCREEN, 26.0, 1893.0, 10.0));
    assert!(!in_corner(SCREEN, 26.0, 1900.0, 26.0));
  }

  #[test]
  fn corner_follows_a_monitor_that_is_not_at_the_origin() {
    let second = Rect::new(1920.0, -200.0, 1280.0, 720.0);
    let r = corner_rect(second, 20.0);
    assert_eq!((r.x, r.y), (3180.0, -200.0));
    assert!(in_corner(second, 20.0, 3199.0, -199.0));
  }

  #[test]
  fn corner_never_exceeds_a_tiny_screen_and_ignores_negative_sizes() {
    let tiny = Rect::new(0.0, 0.0, 10.0, 8.0);
    assert_eq!(corner_rect(tiny, 50.0).w, 8.0);
    assert_eq!(corner_rect(tiny, -5.0).w, 0.0);
  }

  #[test]
  fn a_maximized_ordinary_window_can_be_closed() {
    assert!(can_close(&app("Chrome_WidgetWin_1", "chrome.exe"), 1));
  }

  #[test]
  fn windows_that_are_not_maximized_are_left_alone() {
    let t = Target {
      maximized: false,
      ..app("Notepad", "notepad.exe")
    };
    assert!(!can_close(&t, 1));
  }

  #[test]
  fn desktop_and_shell_windows_are_never_closed() {
    for class in ["Progman", "WorkerW", "Shell_TrayWnd", "shell_secondarytraywnd"] {
      assert!(!can_close(&app(class, "explorer.exe"), 1), "{class}");
    }
  }

  #[test]
  fn nexora_itself_is_never_closed() {
    assert!(!can_close(&app("Nexora", "other.exe"), 4242));
    assert!(!can_close(&app("Whatever", "Nexora.exe"), 1));
    assert!(!can_close(&app("Whatever", "nexora-guard.exe"), 1));
  }
}
