# ADR-0010: Minimize effect names and stored values

Status: Accepted

## Context

The Windows Settings page called stored effect `0` “Funnel”, while the domain model and the implementation in
`window_management::genie` call it “Genie”. The legacy `min_effect` key is persisted as an integer. Renaming the label must
not reinterpret existing settings.

## Decision

Use **Genie**, **Scale**, and **Suck** as the three user-facing effect names. Keep the stable mapping:

| Stored value | Effect |
| ------------ | ------ |
| `0`          | Genie  |
| `1`          | Scale  |
| `2`          | Suck   |

Unknown values continue to fall back to Genie. Parse the old text alias `Funnel` as Genie for compatibility with text
settings, but do not emit it in the Settings UI.

## Consequences

The Windows Settings selector exposes all three domain effects and writes the stable values above. Existing 0.4.0 files
retain their behavior.
