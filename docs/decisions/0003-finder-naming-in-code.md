# 0003 — Code name of the top-bar feature: Finder

Status: accepted · Scope: `crates/core/src/menubar/`, `MenubarSettings`, `crates/app/src/menubar/`, `docs/`

## Context

`docs/feature.md` says code and docs use *Finder* (`FinderSettings`) and only user-visible strings say *Menu Bar*.
The code does the opposite: the feature lives in `core/menubar/`, `MenubarSettings` and `app/menubar/`, and
`feature_checkpoint.md` titles the section *Menubar*. The sidebar of the design prototype calls the group *Finder*.
Three names for one feature make every new setting key (`finder.*` in the design, `menubar_*` on disk) ambiguous.
No `FinderSettings` field exists yet, so renaming now is cheap; after M1 it touches every settings row.

## Decision

Option (a): **follow `feature.md`.** The code name is *Finder*; the UI name stays *Menu Bar* (EN) / *Menu Bar* (ID).

* Rename `core/menubar` to `core/finder`, `MenubarSettings` to `FinderSettings` and `app/menubar` to `app/finder`.
* Keep every former path alive with `pub use` aliases, exactly as ADR-0002 did for `core`
  (`pub use finder as menubar;`, `pub type MenubarSettings = FinderSettings;`). Callers migrate later and the aliases
  are deleted in the same change.
* On-disk keys are **not** renamed (`menubar`, `menubar_style`, `menubar_glass`, `menubar_material`, `clock_24h`,
  `battery_pct`). New keys use the `finder_` prefix.
* The rename lands as one mechanical commit **before** ENG-03 adds any `FinderSettings` field.

## Consequences

* Positive: code, spec, design schema (`finder.*`) and capability IDs (`F-*`) use one word.
* Cost: one mechanical commit touching `core`, `app` and `ui` imports; golden tests prove behavior is unchanged.
* The rename is **not** done by this ADR; it is task ENG-03 (see `TODO.md`).
