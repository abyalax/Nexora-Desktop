//! Finder click handling and menu command execution.

use super::*;

impl Bar {
    pub(super) fn click(&mut self, id: u32) {
        if Self::has_menu(id) {
            if self.open == Some(id) {
                self.close_menu();
            } else {
                self.open_menu(id);
            }
            return;
        }
        match id {
            CTRL => {
                self.close_menu();
                crate::controlcenter::toggle(self.h);
            }
            STAGEI => prefs::update(|st| st.stage = !st.stage),
            SEARCH => crate::spotlight::toggle(),
            WIFI => {
                if let Some((x, _)) = self.rect_of(WIFI) {
                    crate::netmenu::show(
                        &self.comp,
                        crate::netmenu::Which::Wifi,
                        crate::netmenu::Anchor::Below(x as i32, self.h),
                        self.dpi,
                    );
                }
            }
            BATT => launch::open("ms-settings:batterysaver"),
            CLOCK => keys::chord(&[keys::WIN, 0x4E]),
            _ => {}
        }
    }

    pub(super) fn pick(&mut self, cmd: usize) {
        self.close_menu();
        if cmd >= NATIVE_CMD {
            nexora_platform::menus::invoke(self.native_hwnd, (cmd - NATIVE_CMD) as u32);
            return;
        }
        match cmd {
            100 => launch::open("ms-settings:about"),
            101 => launch::open("ms-settings:"),
            102 => launch::open("ms-windows-store:"),
            103 => crate::settings_win::open(&self.comp),
            104 => launch::run("taskmgr.exe", ""),
            105 => crate::launchpad::toggle(),
            106 => crate::mission::toggle(),
            110 => confirm_power(
                "Tidur sekarang?",
                "rundll32.exe",
                "powrprof.dll,SetSuspendState 0,1,0",
            ),
            111 => confirm_power("Mulai ulang komputer sekarang?", "shutdown.exe", "/r /t 0"),
            112 => confirm_power("Matikan komputer sekarang?", "shutdown.exe", "/s /t 0"),
            113 => launch::run("rundll32.exe", "user32.dll,LockWorkStation"),
            114 => confirm_power("Keluar dari akun sekarang?", "shutdown.exe", "/l"),
            800 | 801 => {
                if let Some(rd) = &self.radios {
                    let (k, cur) = if cmd == 800 {
                        (nexora_platform::radio::Kind::Wifi, self.wifi)
                    } else {
                        (nexora_platform::radio::Kind::Bluetooth, self.bt)
                    };
                    let _ = rd
                        .commands
                        .send(nexora_platform::radio::Cmd::Set(k, !cur.unwrap_or(false)));
                }
            }
            802 | 803 => {
                let v = nexora_platform::audio::latest().level;
                services::set_volume((v + if cmd == 802 { 0.1 } else { -0.1 }).clamp(0.0, 1.0));
            }
            804 => services::toggle_mute(),
            805 | 806 => {
                if let Some(v) = nexora_platform::brightness::latest() {
                    services::set_brightness(
                        (v as f32 / 100.0 + if cmd == 805 { 0.1 } else { -0.1 }).clamp(0.0, 1.0),
                    );
                }
            }
            807 => prefs::update(|st| st.stage = !st.stage),
            808 => crate::mission::toggle(),
            809 => crate::launchpad::toggle(),
            810 => crate::settings_win::open(&self.comp),
            811 => launch::open("ms-settings:"),
            200 => crate::settings_win::open(&self.comp),
            201 => self.hide_app(false),
            202 => self.hide_app(true),
            203 => self.show_all(),
            204 => crate::macshortcuts::quit_foreground_app(),
            300 => keys::chord(&[keys::CTRL, 0x4E]),
            301 => keys::chord(&[keys::CTRL, 0x4F]),
            302 => keys::chord(&[keys::CTRL, 0x53]),
            303 => keys::chord(&[keys::CTRL, 0x57]),
            304 => keys::chord(&[keys::CTRL, 0x50]),
            400 => keys::chord(&[keys::CTRL, 0x5A]),
            401 => keys::chord(&[keys::CTRL, 0x59]),
            402 => keys::chord(&[keys::CTRL, 0x58]),
            403 => keys::chord(&[keys::CTRL, 0x43]),
            404 => keys::chord(&[keys::CTRL, 0x56]),
            405 => keys::chord(&[keys::CTRL, 0x41]),
            406 => keys::chord(&[keys::CTRL, 0x46]),
            500 => keys::chord(&[0x7A]),
            501 => keys::chord(&[keys::CTRL, 0xBB]),
            502 => keys::chord(&[keys::CTRL, 0xBD]),
            503 => keys::chord(&[keys::CTRL, 0x30]),
            600 => keys::chord(&[keys::WIN, 0x28]),
            601 => keys::chord(&[keys::WIN, 0x26]),
            700 => keys::chord(&[0x70]),
            _ => {}
        }
    }

    pub(super) fn hide_app(&mut self, others: bool) {
        let fg = focus::foreground();
        let wins = enumerate::list(std::process::id());
        let Some(cur) = wins
            .iter()
            .find(|w| w.hwnd == fg)
            .map(|w| w.path.to_lowercase())
        else {
            return;
        };
        for w in &wins {
            let same = w.path.to_lowercase() == cur;
            if same != others && !focus::is_minimized(w.hwnd) {
                unsafe {
                    let _ = ShowWindow(HWND(w.hwnd as *mut _), SW_MINIMIZE);
                }
                self.hidden.push(w.hwnd);
            }
        }
    }

    pub(super) fn show_all(&mut self) {
        for h in std::mem::take(&mut self.hidden) {
            if focus::is_alive(h) && focus::is_minimized(h) {
                unsafe {
                    let _ = ShowWindow(HWND(h as *mut _), SW_SHOWNOACTIVATE);
                }
            }
        }
    }
}

pub(super) fn confirm_power(question: &'static str, file: &'static str, params: &'static str) {
    let text = prefs::tr(question).to_string();
    std::thread::spawn(move || unsafe {
        use windows::core::HSTRING;
        let r = MessageBoxW(
            None,
            &HSTRING::from(text),
            w!("nexora"),
            MB_YESNO | MB_ICONQUESTION | MB_TOPMOST | MB_SETFOREGROUND,
        );
        if r == IDYES {
            launch::run(file, params);
        }
    });
}
