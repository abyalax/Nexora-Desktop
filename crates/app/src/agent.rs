//! Debug-only local control pipe for runtime verification.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::windows::io::FromRawHandle;
use windows::Win32::Foundation::{CloseHandle, ERROR_PIPE_CONNECTED, GetLastError, HWND};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows::Win32::System::Pipes::{
  ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_TYPE_BYTE, PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use windows::Win32::UI::WindowsAndMessaging::{PostMessageW, WM_APP, WM_COMMAND};
use windows::core::HSTRING;

pub const WM_AGENT_OPEN: u32 = WM_APP + 150;

pub fn start(main_hwnd: HWND) {
  let pid = std::process::id();
  let hwnd = main_hwnd.0 as isize;
  std::thread::Builder::new()
    .name("nexora-agent-pipe".into())
    .spawn(move || serve(pid, HWND(hwnd as *mut _)))
    .ok();
}

fn serve(pid: u32, main_hwnd: HWND) {
  let name = HSTRING::from(format!(r"\\.\pipe\nexora-agent-{pid}"));
  loop {
    let pipe = unsafe {
      CreateNamedPipeW(
        &name,
        PIPE_ACCESS_DUPLEX,
        PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT,
        PIPE_UNLIMITED_INSTANCES,
        8192,
        8192,
        0,
        None,
      )
    };
    if pipe.is_invalid() {
      return;
    }
    let connected =
      unsafe { ConnectNamedPipe(pipe, None) }.is_ok() || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
    if connected {
      // The connected handle is closed when the client disconnects.
      let mut stream = unsafe { std::fs::File::from_raw_handle(pipe.0) };
      if let Ok(reader_stream) = stream.try_clone() {
        let mut reader = BufReader::new(reader_stream);
        let mut line = String::new();
        if reader.read_line(&mut line).is_ok() {
          let response = handle(&line, main_hwnd);
          let _ = writeln!(stream, "{response}");
          let _ = stream.flush();
        }
      }
    } else {
      unsafe {
        let _ = CloseHandle(pipe);
      }
    }
  }
}

fn handle(line: &str, main_hwnd: HWND) -> Value {
  let Ok(cmd) = serde_json::from_str::<Value>(line) else {
    return json!({"ok": false, "error": "invalid JSON"});
  };
  match cmd.get("cmd").and_then(Value::as_str).unwrap_or("") {
    "state" => {
      let deck = crate::dock::agent_state();
      let visor = crate::dynamic_island::agent_state();
      json!({
        "deck": deck,
        "visor": visor,
        "menubar": {"rect": [0, 0, unsafe { GetSystemMetrics(SM_CXSCREEN) }, 48]},
        "animating": deck.as_ref().and_then(|v| v["animating"].as_bool()).unwrap_or(false)
          || visor.as_ref().and_then(|v| v["animating"].as_bool()).unwrap_or(false)
      })
    }
    "wait_idle" => {
      let until = std::time::Instant::now() + std::time::Duration::from_secs(15);
      loop {
        let state = json!({
          "deck": crate::dock::agent_state(),
          "visor": crate::dynamic_island::agent_state(),
        });
        let animating = state["deck"]["animating"].as_bool().unwrap_or(false)
          || state["visor"]["animating"].as_bool().unwrap_or(false);
        if !animating {
          return json!({"ok": true, "idle": true});
        }
        if std::time::Instant::now() >= until {
          return json!({"ok": false, "idle": false, "error": "timeout waiting for animations"});
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
      }
    }
    "quit" => {
      unsafe {
        let _ = PostMessageW(
          Some(main_hwnd),
          WM_COMMAND,
          windows::Win32::Foundation::WPARAM(2),
          windows::Win32::Foundation::LPARAM(0),
        );
      }
      json!({"ok": true})
    }
    "open" => {
      let panel = cmd.get("panel").and_then(Value::as_str).unwrap_or("");
      let (code, page) = match panel {
        "settings" => (1usize, cmd.get("page").and_then(Value::as_i64).unwrap_or(0) as isize),
        "quick-search" => (2, 0),
        "launchpad" => (3, 0),
        "control-center" => (4, 0),
        _ => return json!({"ok": false, "error": "panel must be settings, quick-search, launchpad, or control-center"}),
      };
      unsafe {
        let _ = PostMessageW(
          Some(main_hwnd),
          WM_AGENT_OPEN,
          windows::Win32::Foundation::WPARAM(code),
          windows::Win32::Foundation::LPARAM(page),
        );
      }
      json!({"ok": true})
    }
    "screenshot" => screenshot(&cmd),
    other => json!({"ok": false, "error": format!("unsupported command: {other}")}),
  }
}

fn screenshot(cmd: &Value) -> Value {
  let target = cmd.get("target").and_then(Value::as_str).unwrap_or("");
  let deck = crate::dock::agent_state();
  let visor = crate::dynamic_island::agent_state();
  let rect = match target {
    "deck" => rect4(deck.as_ref().and_then(|v| v.get("rect"))),
    "visor" => rect4(visor.as_ref().and_then(|v| v.get("rect"))),
    "menubar" => Some((0, 0, unsafe { GetSystemMetrics(SM_CXSCREEN) }, 48)),
    "all" => Some((0, 0, unsafe { GetSystemMetrics(SM_CXSCREEN) }, unsafe {
      GetSystemMetrics(SM_CYSCREEN)
    })),
    _ => return json!({"ok": false, "error": "target must be deck, visor, menubar, or all"}),
  };
  let Some(rect) = rect else {
    return json!({"ok": false, "error": format!("{target} is unavailable")});
  };
  let Some(path) = cmd.get("path").and_then(Value::as_str) else {
    return json!({"ok": false, "error": "screenshot requires a path"});
  };
  match nexora_ui::capture::screenshot_png(rect).and_then(|png| std::fs::write(path, &png).map(|_| png.len())) {
    Ok(bytes) => json!({"ok": true, "path": path, "bytes": bytes, "rect": [rect.0, rect.1, rect.2, rect.3]}),
    Err(e) => json!({"ok": false, "error": e.to_string()}),
  }
}

fn rect4(value: Option<&Value>) -> Option<(i32, i32, i32, i32)> {
  let a = value?.as_array()?;
  Some((
    a.first()?.as_i64()? as i32,
    a.get(1)?.as_i64()? as i32,
    a.get(2)?.as_i64()? as i32,
    a.get(3)?.as_i64()? as i32,
  ))
}
