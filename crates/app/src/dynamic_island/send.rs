//! Send-to-phone session: local server, QR code, transfer progress.

use super::*;

impl Notch {
  pub(super) fn start_send(&mut self, paths: Vec<String>) {
    if paths.is_empty() {
      return;
    }
    self.send = None;
    let en = crate::prefs::get().lang == nexora_core::i18n::Lang::En;
    let hw = self.hwnd.0 as isize;
    match nexora_platform::sendsrv::start(&paths, en, move || post(hw, WM_SEND)) {
      Ok(server) => {
        let px = (86.0 * self.dpi).floor();
        let qr = nexora_core::qr::encode(&server.url).and_then(|m| {
          let bmp = nexora_core::qr::render(&m, px as u32, 1);
          self.paint.gfx.surface_from_bgra(px as u32, px as u32, &bmp).ok()
        });
        let names = paths
          .iter()
          .filter_map(|p| {
            std::path::Path::new(p)
              .file_name()
              .map(|n| n.to_string_lossy().to_string())
          })
          .collect();
        let total = paths
          .iter()
          .filter_map(|p| std::fs::metadata(p).ok())
          .map(|m| m.len())
          .sum();
        self.send = Some(SendSess {
          server,
          qr,
          qr_px: px,
          names,
          prog: nexora_platform::sendsrv::Progress {
            total,
            ..Default::default()
          },
          last: Instant::now(),
          done_at: None,
        });
        self.send_err = false;
        timer(self.hwnd, T_SEND, 30_000);
      }
      Err(e) => {
        log::line("kirim", &format!("gagal memulai: {e}"));
        self.send_err = true;
      }
    }
    match self.mode {
      Mode::Drop => self.dispatch(Event::Dropped),
      Mode::Open(Tab::Shelf) => self.refresh(),
      Mode::Open(_) => self.dispatch(Event::SelectTab(Tab::Shelf)),
      _ => {
        self.dispatch(Event::Click);
        self.dispatch(Event::SelectTab(Tab::Shelf));
      }
    }
  }

  pub(super) fn stop_send(&mut self) {
    if self.send.take().is_some() {
      log::line("kirim", "sesi diakhiri");
    }
    self.send_err = false;
    kill(self.hwnd, T_SEND);
    kill(self.hwnd, T_SENDDONE);
    self.refresh();
    if !self.tracking && matches!(self.mode, Mode::Open(_)) {
      self.dispatch(Event::Leave);
    }
  }

  pub(super) fn send_pinned(&self) -> bool {
    self.mode == Mode::Open(Tab::Shelf) && (self.send.is_some() || self.send_err)
  }

  pub(super) fn on_send(&mut self) {
    let Some(s) = self.send.as_mut() else { return };
    let mut changed = false;
    while let Ok(p) = s.server.progress.try_recv() {
      s.prog = p;
      s.last = Instant::now();
      changed = true;
    }
    let n = s.names.len();
    if s.prog.files_done >= n && !s.prog.active && s.done_at.is_none() && s.prog.sent > 0 {
      s.done_at = Some(Instant::now());
      log::line("kirim", &format!("{n} berkas terkirim"));
      timer(
        self.hwnd,
        T_SENDDONE,
        nexora_core::dynamic_island::download::DONE_LINGER_MS,
      );
    } else if s.prog.active {
      s.done_at = None;
    }
    if changed {
      self.refresh();
    }
  }

  pub(super) fn send_compact(&self) -> Option<(f32, bool)> {
    let s = self.send.as_ref()?;
    let done = s
      .done_at
      .is_some_and(|t| t.elapsed().as_millis() < nexora_core::dynamic_island::download::DONE_LINGER_MS as u128);
    (s.prog.active || done).then(|| {
      (
        nexora_core::dynamic_island::send::fraction(s.prog.sent, s.prog.total),
        done && !s.prog.active,
      )
    })
  }
}
