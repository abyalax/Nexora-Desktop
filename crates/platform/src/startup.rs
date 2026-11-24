use std::path::Path;
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
  HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows::core::{PCWSTR, w};

const RUN_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const NAME: PCWSTR = w!("nexora");

pub fn current() -> Option<String> {
  unsafe {
    let mut size = 0u32;
    if RegGetValueW(
      HKEY_CURRENT_USER,
      RUN_KEY,
      NAME,
      RRF_RT_REG_SZ,
      None,
      None,
      Some(&mut size),
    )
    .is_err()
      || size == 0
    {
      return None;
    }
    let mut buf = vec![0u16; (size as usize).div_ceil(2)];
    if RegGetValueW(
      HKEY_CURRENT_USER,
      RUN_KEY,
      NAME,
      RRF_RT_REG_SZ,
      None,
      Some(buf.as_mut_ptr() as *mut _),
      Some(&mut size),
    )
    .is_err()
    {
      return None;
    }
    let n = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..n]))
  }
}

pub fn is_enabled() -> bool {
  current().is_some()
}

pub fn command_for(exe: &Path) -> String {
  format!("\"{}\"", exe.display())
}

pub fn set(enabled: bool, exe: &Path) -> Result<(), String> {
  unsafe {
    if enabled {
      let cmd = command_for(exe);
      let wide: Vec<u16> = cmd.encode_utf16().chain(std::iter::once(0)).collect();
      let err = RegSetKeyValueW(
        HKEY_CURRENT_USER,
        RUN_KEY,
        NAME,
        REG_SZ.0,
        Some(wide.as_ptr() as *const _),
        (wide.len() * 2) as u32,
      );
      if err.is_err() {
        return Err(format!("RegSetKeyValue: {err:?}"));
      }
    } else {
      let err = RegDeleteKeyValueW(HKEY_CURRENT_USER, RUN_KEY, NAME);
      if err.is_err() && err != ERROR_FILE_NOT_FOUND {
        return Err(format!("RegDeleteKeyValue: {err:?}"));
      }
    }
  }
  Ok(())
}

/// Set or delete a Scheduled Task for auto-start (GEN-08 alternate method)
/// Note: Requires COM and Task Scheduler API; currently stubbed for future implementation
pub fn set_scheduled_task(_enabled: bool, _exe: &Path) -> Result<(), String> {
  // TODO: Implement Task Scheduler COM interface
  // This requires using the Task Scheduler 1.0/2.0 COM interfaces
  Err("Scheduled Task support not yet implemented".to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore] // Requires actual Windows registry access
  fn registry_roundtrip() {
    let exe = Path::new("C:\\test\\nexora.exe");
    let before = current();
    set(true, exe).unwrap();
    assert!(is_enabled());
    set(false, exe).unwrap();
    assert!(!is_enabled());
    if let Some(orig) = before {
      set(true, exe).unwrap();
      // restore original
    }
  }
}
