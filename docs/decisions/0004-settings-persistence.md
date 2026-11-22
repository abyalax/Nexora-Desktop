# 0004 — Settings persistence: flat file, table-driven registry

Status: accepted · Scope: `crates/core/src/settings/`, `crates/core/tests/settings_golden.rs`

## Context

`Settings::serialize` is one `format!` with 64 positional placeholders and `apply` is a giant `match` on the key.
Both always write every key, and `tests/settings_golden.rs` requires real 0.4.0 `settings.cfg` files to round-trip
**byte for byte**. Adding one emitted key therefore breaks the golden tests, and every new setting (about 113 are
planned) would edit two long, order-sensitive blocks.

## Decision

1. **Keep the flat `key=value` format** and the file name. No JSON/TOML migration.
2. **Table-driven registry.** One static table, one row per key:
   `Entry { key, kind (bool | int{min,max} | enum | text), default, owner (section), legacy }`.
   Parsing, clamping, defaults and serialization are generated from the table; the giant `match` and the 64-key
   `format!` are removed.
3. **Legacy keys** (the 64 keys of 0.4.0) have `legacy = true`: they are always written, in the original order, so
   0.4.0 files stay byte-identical.
4. **New keys are written only when they differ from their default.** A missing key means "default". The golden
   files therefore never change when a setting is added.
5. **Garbage keeps the old value** (current behavior): unparsable values are ignored, numbers are clamped to range.
6. **`settings_version`:** optional line. Absent means 1 (0.4.0). It is written only when the schema version is > 1
   and is read before other keys, so a later version can migrate old values.
7. Unknown keys are ignored on read and dropped on write (as today).

## Consequences

* Positive: adding a setting is one registry row plus a field; golden tests stay green by construction.
* Cost: ENG-01/02 rewrite `codec.rs` once; the rewrite must pass the existing 262 unit tests, 2 golden tests and 4
  section tests unchanged.
* Every **Co** box in `TODO.md` §5 depends on this ADR.
