# 0005 — List-valued settings live outside `Settings`

Status: accepted · Scope: `crates/core/src/settings/`, `crates/app/src/prefs.rs`

## Context

`Settings` is `Copy` and `prefs::get()` returns it by value; text fields use fixed-size `ShortText`. Six design
settings are lists or free text: excluded programs, running-program blacklist, minimize blacklist, deleted Start Menu
programs, and the two "remove windows" lists. They cannot be stored in a `Copy` struct.

## Options

* (a) Drop `Copy` from `Settings`. Touches every `prefs::get()` call in `app` and `ui`.
* (b) Keep `Settings: Copy`; store lists in a separate `lists.cfg` behind a `ProgramLists` snapshot (`Arc`).
* (c) Fixed-size arrays inside `Settings`. Wastes memory and imposes arbitrary limits.

## Decision

Option (b).

* `core::settings::lists` defines `ProgramLists` (pure data, parsing, case-insensitive matching on executable name).
* `lists.cfg` sits beside `settings.cfg`: one list per `[name]` header, one executable name per line, `#` comments.
  Missing or garbled lines are skipped; a missing file means empty lists.
* The app loads it into an `Arc<ProgramLists>`; `prefs::lists()` clones the `Arc` (cheap). A change swaps the `Arc`;
  consumers read the latest snapshot per use, so no restart is needed.
* Writes are atomic (write temp file, rename).
* Lists are **not** part of the `settings_version` / registry of ADR-0004.

## Consequences

* No change to `prefs::get()` callers. Blocks: WAN-04/05, DLP-05, DMN-09 only get a new accessor.
