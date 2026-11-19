use nexora_core::spotlight::Mode;
use windows::UI::Composition::Compositor;

pub fn init(_comp: &Compositor) -> windows_core::Result<()> {
    Ok(())
}

#[allow(dead_code)]
pub fn shutdown() {}

pub fn toggle() {
    crate::spotlight::toggle_mode(Mode::Apps);
}
