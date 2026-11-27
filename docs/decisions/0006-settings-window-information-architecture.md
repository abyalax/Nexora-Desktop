# 0006 — Settings window information architecture

Status: accepted · Scope: `docs/feature.md` §1, `crates/app/src/settings_win.rs`, `crates/ui`

## Context

The Rust Settings window has 9 pages (Umum, Dock, Menubar, Menu Bar, Tampilan, Scene, Screen Corners, Quick Search,
Widget). The spec (`feature.md` §1) has 20 pages: 6 ungrouped plus the groups Dock (5), Menu Bar (7) and Dynamic
Island (2). Four Rust pages have no place in the spec.

## Decision

Rebuild the sidebar to the spec and add one group for the Rust-only pages.

| Group          | Pages                                                                                   |
| -------------- | --------------------------------------------------------------------------------------- |
| (ungrouped)    | General, Appearance, System Icon Tray, Screen, Stage Manager, Window Animation          |
| Dock           | Look & Behavior, Launchpad, Advanced, Minimize Effects, Window Previews                 |
| Menu Bar       | Look & Behavior, Notifications, Time & Date, Audio, Display, Hardware Status, Battery   |
| Dynamic Island | Look & Behavior, Activities                                                             |
| Extras         | Hot Corners, Quick Search, Widgets                                                      |

Mapping of old pages: Umum → General, Tampilan → Appearance, Scene → Stage Manager, Dock → Dock group, Menubar →
Dynamic Island group, Menu Bar → Menu Bar group, Screen Corners → Extras ▸ Hot Corners, Quick Search → Extras ▸ Quick
Search, Widget → Extras ▸ Widgets.

* A page shows only rows that already have a working effect (rule "no UI row lands without its effect").
* The group label *Menu Bar* is a UI string; the code name is *Finder* (ADR-0003).
* `feature.md` §1 is updated to this table (done in SPEC-01).

## Consequences

* 23 pages in total. `page: u8` is replaced by a page enum in WIN-01. Blocks: WIN-05.
