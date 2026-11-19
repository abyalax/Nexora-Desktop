#![windows_subsystem = "windows"]

mod application;
mod bootstrap;
mod controlcenter;
mod dock;
mod dynamic_island;
mod finder;
mod fx;
mod hotcorners;
mod hud;
mod launchpad;
mod lifecycle;
mod macshortcuts;
mod mainwin;
mod menu;
mod mission;
mod motion;
mod netmenu;
mod paths;
mod popup;
mod prefs;
mod services;
mod settings_win;
mod spotlight;
mod stage;
mod startup;
mod strip;
mod switcher;
mod topmost;
mod widgets;

fn main() {
  bootstrap::run();
}
