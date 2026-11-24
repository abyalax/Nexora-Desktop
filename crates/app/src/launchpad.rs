use nexora_core::launchpad::sources::{Entry, Origin as CoreOrigin, Sources};
use nexora_core::settings::ProgramLists;
use nexora_core::spotlight::Mode;
use nexora_platform::apps::Origin as PlatformOrigin;
use windows::UI::Composition::Compositor;

static CACHE: std::sync::Mutex<Vec<Entry>> = std::sync::Mutex::new(Vec::new());

fn map_origin(o: PlatformOrigin) -> CoreOrigin {
  match o {
    PlatformOrigin::Added => CoreOrigin::Added,
    PlatformOrigin::StartMenu => CoreOrigin::StartMenu,
    PlatformOrigin::Desktop => CoreOrigin::Desktop,
  }
}

pub fn init(_comp: &Compositor) -> windows_core::Result<()> {
  Ok(())
}

#[allow(dead_code)]
pub fn shutdown() {}

pub fn toggle() {
  crate::spotlight::toggle_mode(Mode::Apps);
}

/// Enumerate apps for Launchpad with origin (D-8.2)
#[allow(dead_code)]
pub fn enumerate() -> Vec<Entry> {
  let apps = nexora_platform::apps::snapshot();
  let entries: Vec<Entry> = apps
    .iter()
    .map(|a| Entry {
      name: a.name.clone(),
      origin: map_origin(a.origin),
    })
    .collect();
  *CACHE.lock().unwrap() = entries.clone();
  entries
}

/// Get visible entries filtered by sources and lists (D-8.2)
#[allow(dead_code)]
pub fn visible_entries(sources: Sources, lists: &ProgramLists) -> Vec<Entry> {
  let entries = CACHE.lock().unwrap().clone();
  let refs: Vec<&Entry> = nexora_core::launchpad::sources::visible(&entries, sources, lists);
  refs.into_iter().cloned().collect()
}
