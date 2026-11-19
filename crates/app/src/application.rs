//! The running application: owns what must stay alive for the whole session and pumps messages.

use crate::lifecycle;
use nexora_ui::host::{Host, Surface};
use windows::Win32::Foundation::{HANDLE, HWND};
use windows::Win32::UI::WindowsAndMessaging::{DispatchMessageW, GetMessageW, TranslateMessage, MSG};

pub struct Application {
    // Field order is drop order: release system surfaces first.
    pub(crate) _exit: lifecycle::ExitGuard,
    pub(crate) _instance: HANDLE,
    pub(crate) _hud: Option<HWND>,
    pub(crate) _demo: Option<Surface>,
    pub(crate) _host: Host,
    pub(crate) _main_hwnd: HWND,
}

impl Application {
    /// Runs the Win32 message loop until `WM_QUIT`, then shuts the process down.
    pub fn run(self) -> ! {
        unsafe {
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        lifecycle::shutdown()
    }
}
