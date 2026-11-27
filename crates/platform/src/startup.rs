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

const TASK_NAME: &str = "Nexora Desktop";

/// Set or delete the per-user logon task used by the Scheduled Task start method.
/// `schtasks.exe` is part of Windows and avoids adding a Task Scheduler COM dependency.
pub fn set_scheduled_task(enabled: bool, exe: &Path) -> Result<(), String> {
  set_scheduled_task_named(enabled, exe, TASK_NAME)
}

fn set_scheduled_task_named(enabled: bool, exe: &Path, task_name: &str) -> Result<(), String> {
  let mut command = std::process::Command::new("schtasks.exe");
  if enabled {
    let exe = exe
      .canonicalize()
      .map_err(|e| format!("resolve executable {}: {e}", exe.display()))?;
    // schtasks parses /TR as a command line string. Quote the executable path so spaces
    // remain part of the executable name; no user supplied arguments are inserted.
    let task_command = format!("\"{}\"", exe.display());
    command.args([
      "/Create",
      "/F",
      "/SC",
      "ONLOGON",
      "/TN",
      task_name,
      "/TR",
      &task_command,
      "/RL",
      "LIMITED",
    ]);
  } else {
    command.args(["/Delete", "/F", "/TN", task_name]);
  }
  let output = command.output().map_err(|e| format!("launch schtasks.exe: {e}"))?;
  if output.status.success() || (!enabled && output.status.code() == Some(1) && task_missing(&output)) {
    return Ok(());
  }
  let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
  let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
  Err(format!(
    "schtasks {} failed ({}): {}",
    if enabled { "create" } else { "delete" },
    output.status,
    if stderr.is_empty() { stdout } else { stderr }
  ))
}

fn task_missing(output: &std::process::Output) -> bool {
  let text = format!(
    "{}{}",
    String::from_utf8_lossy(&output.stdout),
    String::from_utf8_lossy(&output.stderr)
  )
  .to_ascii_lowercase();
  text.contains("cannot find the file specified") || text.contains("cannot find the path specified")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  #[ignore = "needs Windows registry access; run with cargo test -p nexora-platform -- --ignored"]
  fn registry_roundtrip() {
    let exe = Path::new("C:\\test\\nexora.exe");
    let before = current();
    set(true, exe).unwrap();
    assert!(is_enabled());
    set(false, exe).unwrap();
    assert!(!is_enabled());
    if let Some(_orig) = before {
      set(true, exe).unwrap();
      // restore original
    }
  }

  #[test]
  #[ignore = "creates a temporary per-user Windows Scheduled Task"]
  fn scheduled_task_roundtrip() {
    let exe = std::env::current_exe().unwrap();
    let task = format!("Nexora Startup Test {}", std::process::id());
    set_scheduled_task_named(true, &exe, &task).unwrap();
    let query = std::process::Command::new("schtasks.exe")
      .args(["/Query", "/TN", &task])
      .output()
      .unwrap();
    assert!(query.status.success(), "{}", String::from_utf8_lossy(&query.stderr));
    set_scheduled_task_named(false, &exe, &task).unwrap();
    let query = std::process::Command::new("schtasks.exe")
      .args(["/Query", "/TN", &task])
      .output()
      .unwrap();
    assert!(!query.status.success(), "temporary task still exists");
  }
}
