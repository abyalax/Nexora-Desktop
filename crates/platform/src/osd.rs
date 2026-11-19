use windows::Win32::UI::WindowsAndMessaging::{FindWindowA, IsWindowVisible, SW_HIDE, ShowWindow};
use windows::core::PCSTR;

pub fn hide_native() {
  unsafe {
    if let Ok(h) = FindWindowA(PCSTR(c"NativeHWNDHost".as_ptr() as *const u8), PCSTR::null()) {
      if IsWindowVisible(h).as_bool() {
        let _ = ShowWindow(h, SW_HIDE);
      }
    }
  }
}
