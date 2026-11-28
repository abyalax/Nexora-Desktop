# Architecture & Development Guidelines

## 1. Project Architecture

This project is a Windows desktop customization application inspired by macOS, built entirely in Rust.

Use a **Cargo Workspace with Modular Architecture**, following separation of concerns and clean dependency boundaries.

### Workspace Crates

| Crate      | Responsibility                                                                            |
| ---------- | ----------------------------------------------------------------------------------------- |
| `app`      | Application entry point, lifecycle, initialization, orchestration, and dependency wiring. |
| `core`     | Domain models, business logic, feature state, policies, and application rules.            |
| `platform` | Windows APIs, Win32, COM, WinRT, shell integration, and system operations.                |
| `ui`       | Rendering, reusable components, feature views, themes, animations, and visual effects.    |

Do not introduce additional crates unless there is a clear architectural or technical justification.

## 2. Dependency Rules

- `app` may depend on all workspace crates.
- `core` must remain independent of UI and OS-specific implementations.
- `platform` owns all Windows-specific operations.
- `ui` handles presentation and consumes core state/models.
- Never place Win32/COM/WinRT implementations inside `core`.
- Never put business logic inside UI components.
- Avoid circular dependencies.
- Prefer explicit interfaces and dependency injection at crate boundaries.

## 3. Directory Ownership

### `crates/app`

Application bootstrap, lifecycle, startup/shutdown, single-instance handling, and feature orchestration.

### `crates/core`

Organize by feature/domain:

- `dock/` — items, pins, layout, behavior, visibility.
- `dynamic_island/` — state, events, layout, interactions.
- `taskbar/` — state, items, layout.
- `panels/` — control center, notifications, media, calendar.
- `launchpad/` — application catalog and categories.
- `window_management/` — window state and policies.
- `settings/` — schemas, defaults, profiles.
- `shared/` — common types, errors, and domain events.

### `crates/platform`

Organize by OS capability:

- `windows/` — Win32, shell, taskbar, windows, monitors, registry.
- `system/` — audio, brightness, Bluetooth, Wi-Fi, battery, power.
- `media/` — media sessions, metadata, playback.
- `filesystem/` — files, icons, recycle bin.
- `input/` — keyboard, mouse, hotkeys.

### `crates/ui`

Organize by presentation responsibility:

- `components/` — reusable UI primitives.
- `features/` — Dock, Dynamic Island, Control Center, Launchpad, etc.
- `effects/` — blur, glass, shadows, backdrop.
- `layouts/` — reusable panel and window layouts.
- `theme.rs` — design tokens and theming.
- `animation.rs` — shared animation utilities.
- `renderer.rs` — rendering abstraction.

## 4. Rust Conventions

- Use the Rust edition set in the workspace `Cargo.toml` (currently 2024; see `docs/decisions/0009-edition-2021.md`, migrated in M7).
- Follow standard Rust naming conventions: `snake_case` for modules/functions, `PascalCase` for types.
- Prefer small, cohesive modules over monolithic files.
- Use `mod.rs` consistently for directory-based modules.
- Keep `main.rs` minimal; delegate initialization to `bootstrap.rs`.
- Use `thiserror` for library errors.
- Use `tracing` for structured logging.
- Use `serde` for serialization where needed.
- Avoid unnecessary cloning, allocations, and shared mutable state.
- Prefer ownership and borrowing over excessive `Arc<Mutex<T>>`.
- Use `Result` for recoverable errors; avoid unnecessary `unwrap()` and `expect()` in production code.
- Separate persistent configuration from runtime state.
- Avoid one file with more 500 line of code

## 5. Feature Organization

Each feature must maintain a clear separation between:

1. Domain logic (`core`)
2. OS integration (`platform`)
3. Presentation (`ui`)

Example:

```text
core/dock/
├── mod.rs
├── items.rs
├── layout.rs
└── visibility.rs

platform/windows/
└── shell.rs

ui/features/dock/
├── mod.rs
├── view.rs
├── item.rs
└── preview.rs
```

Do not duplicate domain logic across layers.

## 6. Windhawk Reference Integration

Windhawk Mods are reference implementations, not direct Rust dependencies.

Keep reference sources isolated:

```Shell
└── 📁mods
    └── 📁local@dynamic-island-for-windows
        ├── analysis-summary.json
        ├── functions.csv
        ├── functions.json
        ├── includes.csv
        ├── lifecycle-symbols.csv
        ├── README-ANALYSIS.md
        ├── repository-files.csv
        ├── variable-candidates.csv
        ├── windhawk-settings.csv
        ├── windows-api-usage.csv
    └── 📁local@mac-island-dock-fork
        ├── analysis-summary.json
        ├── functions.csv
        ├── functions.json
        ├── includes.csv
        ├── lifecycle-symbols.csv
        ├── MODULES.md
        ├── README-ANALYSIS.md
        ├── repository-files.csv
        ├── variable-candidates.csv
        ├── windhawk-settings.csv
        ├── windows-api-usage.csv
    ├── local@dynamic-island-for-windows.wh.cpp
    ├── local@mac-island-dock-fork.wh.cpp
    └── README.md
```

Rules:

- Never manually edit generated files.
- Preserve original source references.
- Identical inputs must produce identical outputs.
- Keep source transformation logic separate from application code.
- Port functionality into native Rust modules; do not blindly translate C++ file structures.

Map reference code by responsibility, not original filename.

## 7. Development Standards

Before considering a task complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets
cargo test --workspace
cargo check --workspace
```

- Add unit tests for domain logic.
- Add integration tests for cross-crate behavior.
- Keep platform-specific code isolated and testable.
- Do not introduce dependencies without justification.
- Do not silently change public interfaces or architectural boundaries.

## 8. AI Agent Instructions

### Runtime testing (Windows only)

- In PowerShell set $env:NACRE_AGENT = "1" and $env:NACRE_CONFIG_DIR = "target\cfg-test", then run cargo run --features agent-debug -- --keep-taskbar --exit-after 60.
- Use nexora-ctl --pid <PID> state and wait_idle before screenshots or layout assertions.
- Exit through nexora-ctl --pid <PID> quit; do not terminate the process.
- Keep injected input inside a Nexora window rectangle.
- Runtime layout checks cover one primary monitor.

When implementing or modifying code:

1. Inspect existing modules and conventions before creating files.
2. Identify the correct crate and responsibility before implementation.
3. Reuse existing abstractions instead of duplicating functionality.
4. Keep changes scoped to the requested feature.
5. Do not create speculative modules or abstractions without a concrete use case.
6. Update `mod.rs` and Cargo dependencies when necessary.
7. Preserve existing functionality unless explicitly instructed otherwise.
8. Run relevant formatting, checks, and tests after modifications.
9. Document significant architectural decisions in `docs/decisions/`.
10. If a requirement conflicts with these guidelines, explain the conflict before introducing architectural changes.

**Core principle:** Keep the architecture modular, dependencies explicit, platform-specific code isolated, and feature implementations cohesive. Prefer maintainable solutions over premature abstraction.
