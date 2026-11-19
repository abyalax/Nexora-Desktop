use nexora_core::visibility::policy::Mode;
use nexora_ui::glass::Tier;

/// Command-line flags. Flag spellings are part of the public CLI and of existing scripts, so they
/// keep their original words (`--no-dock`, `--no-island`, ...).
pub struct Args {
  pub hide_taskbar: bool,
  pub demo_surface: bool,
  pub no_dock: bool,
  pub no_island: bool,
  pub no_hud: bool,
  pub appbar: bool,
  pub dock_mode: Option<Mode>,
  pub island_mode: Option<Mode>,
  pub keep_taskbar: bool,
  pub open_settings: bool,
  pub open_widgets: bool,
  pub add_app: Option<String>,
  pub set_icon: Option<(String, String)>,
  pub glass: Option<Tier>,
  pub exit_after: Option<u32>,
}

fn flag_value(v: &[String], name: &str) -> Option<String> {
  v.iter().position(|a| a == name).and_then(|i| v.get(i + 1)).cloned()
}

impl Args {
  pub fn parse() -> Args {
    let v: Vec<String> = std::env::args().skip(1).collect();
    Args {
      hide_taskbar: v.iter().any(|a| a == "--hide-taskbar"),
      demo_surface: v.iter().any(|a| a == "--demo-surface"),
      no_dock: v.iter().any(|a| a == "--no-dock"),
      no_island: v.iter().any(|a| a == "--no-island"),
      no_hud: v.iter().any(|a| a == "--no-hud"),
      appbar: v.iter().any(|a| a == "--appbar"),
      island_mode: flag_value(&v, "--island-mode").and_then(|s| Mode::parse(&s)),
      dock_mode: flag_value(&v, "--dock-mode").and_then(|s| Mode::parse(&s)),
      add_app: flag_value(&v, "--add-app"),
      set_icon: v
        .iter()
        .position(|a| a == "--set-icon")
        .and_then(|i| Some((v.get(i + 1)?.clone(), v.get(i + 2)?.clone()))),
      open_settings: v.iter().any(|a| a == "--open-settings"),
      open_widgets: v.iter().any(|a| a == "--open-widgets"),
      keep_taskbar: v.iter().any(|a| a == "--keep-taskbar"),
      glass: flag_value(&v, "--glass").and_then(|s| match s.as_str() {
        "full" => Some(Tier::Full),
        "light" => Some(Tier::Light),
        "solid" => Some(Tier::Solid),
        _ => None,
      }),
      exit_after: v
        .iter()
        .position(|a| a == "--exit-after")
        .and_then(|i| v.get(i + 1))
        .and_then(|s| s.parse().ok()),
    }
  }
}
