//! `--demo-surface`: a bare glass strip used to eyeball the compositor without any feature running.

use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::*;

unsafe extern "system" fn demo_proc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    DefWindowProcW(h, m, w, l)
}

pub fn create(host: &nexora_ui::host::Host) -> windows::core::Result<nexora_ui::host::Surface> {
    unsafe {
        let hi: HINSTANCE = GetModuleHandleW(None)?.into();
        let class = w!("NacreDemo");
        RegisterClassW(&WNDCLASSW {
            lpfnWndProc: Some(demo_proc),
            hInstance: hi,
            lpszClassName: class,
            ..Default::default()
        });
        let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        let (w_, h_) = (520, 72);
        let hwnd = CreateWindowExW(
            WS_EX_NOREDIRECTIONBITMAP | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class,
            w!("nacre-demo"),
            WS_POPUP | WS_VISIBLE,
            sw / 2 - w_ / 2,
            sh - 12 - h_,
            w_,
            h_,
            None,
            None,
            Some(hi),
            None,
        )?;
        let surface = host.attach(hwnd, (w_ as f32, h_ as f32))?;
        nexora_ui::glass::add_panel(
            &host.compositor,
            &surface.root,
            nexora_ui::glass::GlassParams::light((w_ as f32, h_ as f32), 24.0),
        )?;
        Ok(surface)
    }
}
