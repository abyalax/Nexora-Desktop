//! Connected display inventory for settings and display-aware surfaces.

use windows::Win32::Foundation::{LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{
  DISPLAY_DEVICEW, EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
  MONITORINFOEXW,
};
use windows::core::{BOOL, PCWSTR};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monitor {
  /// Windows display device path (stable across reconnects when the same display is used).
  pub id: String,
  /// Friendly display name reported by the display driver.
  pub name: String,
  /// Current display bounds in virtual-screen coordinates.
  pub bounds: (i32, i32, i32, i32),
  /// Work area excluding the Windows taskbar and other app bars.
  pub work_area: (i32, i32, i32, i32),
  pub primary: bool,
}

fn rect(r: RECT) -> (i32, i32, i32, i32) {
  (r.left, r.top, r.right, r.bottom)
}

unsafe fn friendly_name(id: &str) -> String {
  unsafe {
    let id: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
    let mut device = DISPLAY_DEVICEW {
      cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
      ..Default::default()
    };
    if !EnumDisplayDevicesW(PCWSTR(id.as_ptr()), 0, &mut device, 0).as_bool() {
      return String::new();
    }
    let end = device
      .DeviceString
      .iter()
      .position(|c| *c == 0)
      .unwrap_or(device.DeviceString.len());
    String::from_utf16_lossy(&device.DeviceString[..end])
  }
}

unsafe extern "system" fn collect(monitor: HMONITOR, _dc: HDC, _rect: *mut RECT, data: LPARAM) -> BOOL {
  unsafe {
    let out = &mut *(data.0 as *mut Vec<Monitor>);
    let mut info = MONITORINFOEXW {
      monitorInfo: MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
        ..Default::default()
      },
      ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info as *mut MONITORINFOEXW as *mut MONITORINFO).as_bool() {
      return BOOL(1);
    }
    let end = info
      .szDevice
      .iter()
      .position(|c| *c == 0)
      .unwrap_or(info.szDevice.len());
    let id = String::from_utf16_lossy(&info.szDevice[..end]);
    if !id.is_empty() {
      out.push(Monitor {
        name: friendly_name(&id),
        id,
        bounds: rect(info.monitorInfo.rcMonitor),
        work_area: rect(info.monitorInfo.rcWork),
        primary: info.monitorInfo.dwFlags & 1 != 0,
      });
    }
    BOOL(1)
  }
}

/// Enumerate active displays in Windows' current monitor order.
pub fn list() -> Vec<Monitor> {
  let mut out = Vec::new();
  unsafe {
    let _ = EnumDisplayMonitors(
      None,
      None,
      Some(collect),
      LPARAM(&mut out as *mut Vec<Monitor> as isize),
    );
  }
  out
}
