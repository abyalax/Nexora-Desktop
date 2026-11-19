use windows::Win32::UI::Shell::{SHQUERYRBINFO, SHQueryRecycleBinW};
use windows::core::PCWSTR;

pub fn is_full() -> bool {
  let mut info = SHQUERYRBINFO {
    cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
    ..Default::default()
  };
  unsafe { SHQueryRecycleBinW(PCWSTR::null(), &mut info) }.is_ok() && info.i64NumItems > 0
}
