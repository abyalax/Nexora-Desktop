//! Nexora core: domain models, business rules and feature state.
//!
//! This crate has no UI and no operating-system dependencies (see `docs/architecture.md` §3.2). Code is organised by
//! feature; the three headline features are [`dock`], [`finder`] (the menu bar) and [`dynamic_island`].

pub mod dock;
pub mod dynamic_island;
pub mod finder;
pub mod launchpad;
pub mod panels;
pub mod settings;
pub mod shared;
pub mod visibility;
pub mod window_management;

// Compatibility re-exports.
//
// Modules moved into feature folders keep resolving under their former top-level paths so that `app`, `ui` and
// `platform` compile unchanged. Migrate callers to the canonical paths and delete these lines in the same change
// that updates the callers (see `docs/decisions/0002-regroup-core-by-feature.md`).
// `menubar` is the former name of `finder` (ADR-0003); on-disk keys keep the `menubar_` prefix.
pub use finder as menubar;
pub use launchpad::{categories as appcat, search, spotlight};
pub use panels::{control_center as controls, hud, network as netlist, widgets};
pub use settings::preview;
pub use shared::{i18n, iconplate, logo, symbols};
pub use window_management::{genie, marker, mission, stage};
// `dock::trash` and `dynamic_island::qr` are also exposed at their former top-level paths.
pub use dock::trash;
pub use dynamic_island::qr;
