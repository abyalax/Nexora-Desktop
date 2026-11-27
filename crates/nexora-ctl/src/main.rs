use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext};
use windows::Win32::UI::Input::KeyboardAndMouse::{
  INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
  MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEEVENTF_VIRTUALDESK, MOUSEINPUT, SendInput, VIRTUAL_KEY, VkKeyScanW,
};
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

fn main() {
  if let Err(e) = run() {
    eprintln!("nexora-ctl: {e}");
    std::process::exit(1);
  }
}

fn run() -> Result<(), String> {
  unsafe {
    let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
  }
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  let (pid, i) = if args.first().is_some_and(|a| a == "--pid") {
    (number(args.get(1).cloned(), "pid")? as u32, 2)
  } else {
    (
      std::env::var("NACRE_AGENT_PID")
        .map_err(|_| "set NACRE_AGENT_PID or pass --pid <pid>".to_string())?
        .parse()
        .map_err(|_| "invalid NACRE_AGENT_PID".to_string())?,
      0,
    )
  };
  let command = args.get(i).ok_or_else(usage)?.as_str();
  let tail = &args[i + 1..];
  match command {
    "move" => {
      let (x, y) = (number(tail.first().cloned(), "x")?, number(tail.get(1).cloned(), "y")?);
      ensure_inside(pid, x, y)?;
      mouse_move(x, y)
    }
    "click" => {
      let (x, y) = cursor_position()?;
      ensure_inside(pid, x, y)?;
      mouse_click()
    }
    "key" => {
      let (x, y) = cursor_position()?;
      ensure_inside(pid, x, y)?;
      key(tail.first().ok_or_else(usage)?)
    }
    "state" | "wait_idle" | "quit" | "screenshot" | "open" => pipe_command(pid, command, tail),
    _ => Err(usage()),
  }
}

fn pipe_command(pid: u32, command: &str, rest: &[String]) -> Result<(), String> {
  let mut request = json!({"cmd": command});
  if command == "screenshot" {
    request["target"] = rest.first().map_or(Value::Null, |s| json!(s));
    request["path"] = rest.get(1).map_or(Value::Null, |s| json!(s));
  }
  if command == "open" {
    request["panel"] = rest.first().map_or(Value::Null, |s| json!(s));
    request["page"] = rest
      .get(1)
      .and_then(|s| s.parse::<u8>().ok())
      .map_or(Value::Null, |p| json!(p));
  }
  let value = pipe_request(pid, request)?;
  println!("{}", serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?);
  if value.get("ok") == Some(&Value::Bool(false)) {
    return Err(value["error"].as_str().unwrap_or("command failed").to_string());
  }
  Ok(())
}

fn pipe_request(pid: u32, request: Value) -> Result<Value, String> {
  let pipe = format!(r"\\.\pipe\nexora-agent-{pid}");
  let mut file = std::fs::OpenOptions::new()
    .read(true)
    .write(true)
    .open(&pipe)
    .map_err(|e| format!("cannot connect to {pipe}: {e}"))?;
  writeln!(file, "{request}").map_err(|e| e.to_string())?;
  let mut response = String::new();
  BufReader::new(file)
    .read_line(&mut response)
    .map_err(|e| e.to_string())?;
  serde_json::from_str(&response).map_err(|e| format!("invalid response: {e}"))
}

fn ensure_inside(pid: u32, x: i32, y: i32) -> Result<(), String> {
  let state = pipe_request(pid, json!({"cmd":"state"}))?;
  let inside = ["deck", "visor", "menubar"].iter().any(|key| {
    let Some(a) = state.get(*key).and_then(|v| v.get("rect")).and_then(Value::as_array) else {
      return false;
    };
    if a.len() != 4 {
      return false;
    }
    let (Some(l), Some(t), Some(w), Some(h)) = (a[0].as_i64(), a[1].as_i64(), a[2].as_i64(), a[3].as_i64()) else {
      return false;
    };
    x >= l as i32 && y >= t as i32 && x < (l + w) as i32 && y < (t + h) as i32
  });
  if inside {
    Ok(())
  } else {
    Err("input is outside the current Nexora window rectangles".into())
  }
}

fn cursor_position() -> Result<(i32, i32), String> {
  let mut p = POINT::default();
  if unsafe { GetCursorPos(&mut p) }.is_ok() {
    Ok((p.x, p.y))
  } else {
    Err("GetCursorPos failed".into())
  }
}

fn mouse_move(x: i32, y: i32) -> Result<(), String> {
  unsafe {
    let (w, h) = (
      GetSystemMetrics(SM_CXSCREEN).max(1) - 1,
      GetSystemMetrics(SM_CYSCREEN).max(1) - 1,
    );
    let (ax, ay) = ((x.clamp(0, w) * 65535 / w), (y.clamp(0, h) * 65535 / h));
    send_mouse(MOUSEINPUT {
      dx: ax,
      dy: ay,
      dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE | MOUSEEVENTF_VIRTUALDESK,
      ..Default::default()
    })
  }
}

fn mouse_click() -> Result<(), String> {
  unsafe {
    send_mouse(MOUSEINPUT {
      dwFlags: MOUSEEVENTF_LEFTDOWN,
      ..Default::default()
    })?;
    send_mouse(MOUSEINPUT {
      dwFlags: MOUSEEVENTF_LEFTUP,
      ..Default::default()
    })
  }
}

unsafe fn send_mouse(input: MOUSEINPUT) -> Result<(), String> {
  let item = INPUT {
    r#type: INPUT_MOUSE,
    Anonymous: INPUT_0 { mi: input },
  };
  if unsafe { SendInput(&[item], std::mem::size_of::<INPUT>() as i32) } == 1 {
    Ok(())
  } else {
    Err("SendInput failed".into())
  }
}

fn key(text: &str) -> Result<(), String> {
  let code = text
    .parse::<u16>()
    .ok()
    .or_else(|| {
      let mut chars = text.encode_utf16();
      let ch = chars.next()?;
      if chars.next().is_some() {
        return None;
      }
      let mapped = unsafe { VkKeyScanW(ch) };
      (mapped != -1).then_some(mapped as u16)
    })
    .ok_or_else(|| "key must be a virtual key number or one character".to_string())?;
  unsafe {
    for flags in [Default::default(), KEYEVENTF_KEYUP] {
      let item = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
          ki: KEYBDINPUT {
            wVk: VIRTUAL_KEY(code),
            dwFlags: flags,
            ..Default::default()
          },
        },
      };
      if SendInput(&[item], std::mem::size_of::<INPUT>() as i32) != 1 {
        return Err("SendInput failed".into());
      }
    }
  }
  Ok(())
}

fn number(value: Option<String>, label: &str) -> Result<i32, String> {
  value.ok_or_else(usage)?.parse().map_err(|_| format!("invalid {label}"))
}

fn usage() -> String {
  "usage: nexora-ctl [--pid PID] state|wait_idle|quit|screenshot TARGET PATH|open PANEL [PAGE]|move X Y|click|key KEY"
    .into()
}
