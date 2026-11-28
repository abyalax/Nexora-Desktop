# 0004 — Settings persistence: flat file, table-driven registry

Status: accepted · Scope: `crates/core/src/settings/`, `crates/core/tests/settings_golden.rs`

## Context

Before the registry migration, `Settings::serialize` was one `format!` with 64 positional placeholders and `parse` was
a giant `match` on the key. `tests/settings_golden.rs` requires real 0.4.0 `settings.cfg` files to round-trip
**byte for byte**. Adding an emitted key therefore breaks the golden tests, and every new setting (about 113 were
planned) would edit two long, order-sensitive blocks.

## Decision

1. **Keep the flat `key=value` format** and the file name. No JSON/TOML migration.
2. **Table-driven registry.** `settings::registry::SETTINGS_REGISTRY` is the single registry entry point. Its frozen
   legacy rows and post-0.4 extension rows each have one descriptor per key with kind, default, owner and legacy
   status; `all_metadata()` exposes them as one view. Parser and writer callbacks live on legacy rows, while the
   extension row macro generates parsing, clamping, defaulting and non-default serialization. The key `match` and
   positional 64-key `format!` are removed from the flat codec.
3. **Legacy keys** (the 64 keys of 0.4.0) have `legacy = true`: they are always written, in the original order, so
   0.4.0 files stay byte-identical.
4. **New keys are written only when they differ from their default.** A missing key means "default". The golden
   files therefore never change when a setting is added.
5. **Garbage keeps the old value** (current behavior): unparsable values are ignored, numbers are clamped to range.
6. **`settings_version`:** optional line. Absent means 1 (0.4.0). It is written only when the schema version is > 1
   and is read before other keys, so a later version can migrate old values.
7. Unknown keys are ignored on read and dropped on write (as today).
8. `DockConfig` stays a composed Dock-domain behavior value; it does not replace or get embedded wholesale in
   `settings::DockSettings`. `DockSettings` remains the projection of the persisted `Settings` aggregate, while
   `DockConfig` owns the Dock effect/policy vocabulary and its standalone round-trip helpers. App wiring must project
   the persisted Dock rows explicitly when a consumer needs those policies; duplicate persisted values must not be
   introduced.

## Consequences

* Positive: adding a setting is one registry row plus a field; golden tests stay green by construction.
* New files emit `settings_version=2`. Unversioned 0.4.0 files read as version 1 and remain byte-identical when saved;
  the version is scanned before all other keys.
* `settings::registry` checks legacy order/defaults against the 0.4.0 fixture, key uniqueness and malformed-value
  behavior. Golden, extension, section and upgrade tests exercise both registry generations.
* Every **Co** box in `TODO.md` §5 depends on this ADR.
