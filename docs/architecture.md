# Architecture and Project Structure

This document defines the architecture, repository layout, crate boundaries, Rust conventions, and implementation sequence for **Nexora Desktop**.
The structure is a target architecture, not a requirement to create every directory immediately. Add modules when they have a clear responsibility and a concrete implementation.

## 1. Architecture
### Modular Cargo workspace

Use a **Cargo workspace with a modular architecture**, guided by Clean Architecture and separation of concerns.

| Crate      | Responsibility                                                                    |
| ---------- | --------------------------------------------------------------------------------- |
| `app`      | Application entry point, lifecycle management, composition, and dependency wiring |
| `core`     | Domain models, business rules, application logic, and runtime state               |
| `platform` | Windows APIs, operating-system integration, and system-level operations           |
| `ui`       | Rendering, reusable components, widgets, and visual effects                       |

### Dependency boundaries

The `app` crate acts as the composition root. It initializes the application and wires together the core, platform, and UI layers.
- **`app`** — application entry point and orchestration
- **`core`** — domain logic and application rules
- **`platform`** — Windows APIs, OS capabilities, and system integration
- **`ui`** — presentation and rendering
Treat this as a conceptual architecture; the dependency rules below are authoritative.

**Dependency rules:**

- `core` must not depend on `ui` or concrete Windows implementations.
- `platform` must not depend on `app`.
- `ui` may depend on `core` abstractions and domain types, but should not own business rules.
- `app` may depend on all other crates to compose the application.
- If core logic needs an OS capability, define an appropriate abstraction at the boundary and provide its implementation in `platform`.

## 2. Repository structure

The following is the proposed initial project structure:

```Shell
nexora-desktop/
│
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .gitignore
├── README.md
├── LICENSE
│
├── crates/
│   │
│   ├── app/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── bootstrap.rs
│   │       ├── application.rs
│   │       ├── lifecycle.rs
│   │       │
│   │       ├── startup/
│   │       │   ├── mod.rs
│   │       │   ├── single_instance.rs
│   │       │   └── recovery.rs
│   │       │
│   │       └── commands/
│   │           ├── mod.rs
│   │           └── registry.rs
│   │
│   ├── core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── dock/
│   │       │   ├── mod.rs
│   │       │   ├── items.rs
│   │       │   ├── layout.rs
│   │       │   ├── behavior.rs
│   │       │   ├── pins.rs
│   │       │   └── visibility.rs
│   │       │
│   │       ├── dynamic_island/
│   │       │   ├── mod.rs
│   │       │   ├── state.rs
│   │       │   ├── events.rs
│   │       │   ├── layout.rs
│   │       │   └── interactions.rs
│   │       │
│   │       ├── taskbar/
│   │       │   ├── mod.rs
│   │       │   ├── state.rs
│   │       │   ├── items.rs
│   │       │   └── layout.rs
│   │       │
│   │       ├── panels/
│   │       │   ├── mod.rs
│   │       │   ├── control_center.rs
│   │       │   ├── notification.rs
│   │       │   ├── media.rs
│   │       │   └── calendar.rs
│   │       │
│   │       ├── launchpad/
│   │       │   ├── mod.rs
│   │       │   ├── apps.rs
│   │       │   └── categories.rs
│   │       │
│   │       ├── window_management/
│   │       │   ├── mod.rs
│   │       │   ├── state.rs
│   │       │   └── policies.rs
│   │       │
│   │       ├── settings/
│   │       │   ├── mod.rs
│   │       │   ├── schema.rs
│   │       │   ├── defaults.rs
│   │       │   └── profiles.rs
│   │       │
│   │       └── shared/
│   │           ├── mod.rs
│   │           ├── events.rs
│   │           ├── errors.rs
│   │           └── types.rs
│   │
│   ├── platform/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       │
│   │       ├── windows/
│   │       │   ├── mod.rs
│   │       │   ├── window.rs
│   │       │   ├── process.rs
│   │       │   ├── monitor.rs
│   │       │   ├── display.rs
│   │       │   ├── shell.rs
│   │       │   ├── taskbar.rs
│   │       │   ├── registry.rs
│   │       │   └── notifications.rs
│   │       │
│   │       ├── system/
│   │       │   ├── mod.rs
│   │       │   ├── audio.rs
│   │       │   ├── brightness.rs
│   │       │   ├── bluetooth.rs
│   │       │   ├── wifi.rs
│   │       │   ├── battery.rs
│   │       │   └── power.rs
│   │       │
│   │       ├── media/
│   │       │   ├── mod.rs
│   │       │   ├── session.rs
│   │       │   ├── metadata.rs
│   │       │   └── playback.rs
│   │       │
│   │       ├── filesystem/
│   │       │   ├── mod.rs
│   │       │   ├── files.rs
│   │       │   ├── icons.rs
│   │       │   └── recycle_bin.rs
│   │       │
│   │       └── input/
│   │           ├── mod.rs
│   │           ├── keyboard.rs
│   │           ├── mouse.rs
│   │           └── hotkeys.rs
│   │
│   └── ui/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── renderer.rs
│           ├── window.rs
│           ├── theme.rs
│           ├── animation.rs
│           │
│           ├── components/
│           │   ├── mod.rs
│           │   ├── button.rs
│           │   ├── icon.rs
│           │   ├── tooltip.rs
│           │   ├── popup.rs
│           │   ├── slider.rs
│           │   └── text.rs
│           │
│           ├── features/
│           │   ├── mod.rs
│           │   ├── dock/
│           │   │   ├── mod.rs
│           │   │   ├── view.rs
│           │   │   ├── item.rs
│           │   │   └── preview.rs
│           │   │
│           │   ├── dynamic_island/
│           │   │   ├── mod.rs
│           │   │   ├── view.rs
│           │   │   └── animations.rs
│           │   │
│           │   ├── control_center/
│           │   │   ├── mod.rs
│           │   │   └── view.rs
│           │   │
│           │   └── launchpad/
│           │       ├── mod.rs
│           │       └── view.rs
│           │
│           ├── effects/
│           │   ├── mod.rs
│           │   ├── blur.rs
│           │   ├── glass.rs
│           │   ├── shadow.rs
│           │   └── backdrop.rs
│           │
│           └── layouts/
│               ├── mod.rs
│               └── panel.rs
│
├── assets/
│   ├── icons/
│   ├── images/
│   ├── fonts/
│   ├── themes/
│   └── animations/
│
├── config/
│   ├── default.toml
│   └── schemas/
│
├── resources/
│   ├── windows/
│   └── localization/
│
├── scripts/
│   ├── windhawk/
│   │   ├── extract.ps1
│   │   ├── merge.ps1
│   │   └── validate.ps1
│   │
│   └── development/
│       ├── setup.ps1
│       └── build.ps1
│
├── docs/
│   ├── architecture/
│   ├── features/
│   ├── decisions/
│   └── references/
│
└── tests/
    ├── integration/
    └── fixtures/
```

This is the intended direction, not a mandate to create every file upfront. Introduce modules only when their responsibilities and implementations are sufficiently clear.

## 3. Crate responsibilities

This section defines where code belongs and helps prevent responsibilities from drifting across crate boundaries.

### 3.1 `app` — Application orchestration

`app` is the application entry point and composition layer.

Responsibilities:

- Initialize the application.
- Construct and wire dependencies.
- Run the application lifecycle.
- Enforce single-instance behavior.
- Connect the UI to core logic and platform services.
- Coordinate shutdown and recovery.

Example:

```rust
fn main() {
    let application = bootstrap::initialize();
    application.run();
}
```

Keep `main.rs` intentionally small. Do not place Dock, Dynamic Island, or direct Windows API implementations in it.

### 3.2 `core` — Domain and application logic

`core` contains the application's central rules, domain models, and state. It should remain independent of rendering details and concrete operating-system APIs.

Examples of domain responsibilities:

**Dock**

- Manage Dock items.
- Pin and unpin applications.
- Order items.
- Define Dock positioning rules.
- Manage auto-hide policies.
- Track Dock visibility state.

**Dynamic Island**

- Manage Island state.
- Handle application-level events.
- Define expansion and collapse behavior.
- Track notification state.
- Track timer state.

**Settings**

- Define configuration schemas.
- Store user preferences.
- Manage profiles.
- Define default settings.

Example:

```rust
pub struct DockItem {
    pub id: String,
    pub name: String,
    pub pinned: bool,
    pub position: usize,
}

pub struct DockState {
    pub items: Vec<DockItem>,
    pub visible: bool,
}
```

The core crate must not know how the Dock is rendered or how Windows creates and positions its window.

### 3.3 `platform` — Windows and system integration

Because Nexora Desktop targets Windows desktop customization, this crate owns interactions with operating-system capabilities, including:

- Win32 API
- Windows Shell
- COM
- Windows Runtime (WinRT)
- Window enumeration and positioning
- Process management
- System notifications
- Media Session API
- Audio endpoints
- Display configuration

For example, retrieving the applications currently running should follow a boundary such as:

```text
UI Dock
    |
    v
Core Dock
    |
    v
Platform Windows
    |
    v
Windows API
    |
    v
Running Applications
```

Do not implement Win32 calls directly inside `core/dock`. Put concrete Windows implementations in `platform/windows`.

### 3.4 `ui` — Rendering and visual system

All presentation-related code belongs in `ui`.

| Module       | Responsibility                            |
| ------------ | ----------------------------------------- |
| `components` | Reusable UI components                    |
| `features`   | Feature-specific views and presentation   |
| `effects`    | Blur, glass, shadow, and backdrop effects |
| `layouts`    | Panel and window layouts                  |
| `animation`  | Transitions and animation system          |
| `theme`      | Design tokens and theming                 |

A feature may have both domain logic and presentation code. Keep those responsibilities separate.

Example:

```text
core/dock/
├── items.rs
├── layout.rs
└── visibility.rs

ui/features/dock/
├── view.rs
├── item.rs
└── preview.rs
```

`core/dock/layout.rs` determines the logical arrangement of Dock items. `ui/features/dock/view.rs` renders that arrangement on screen.

## 4. Mapping Windhawk references to the new architecture

Use the existing Windhawk implementation as a reference, then reorganize each responsibility according to its role in Nexora Desktop.

| Windhawk reference | Rust destination                                       |
| ------------------ | ------------------------------------------------------ |
| `foundation/`      | `core/shared/`, `platform/windows/`                    |
| `i18n/`            | `resources/localization/`                              |
| `island/`          | `core/dynamic_island/` + `ui/features/dynamic_island/` |
| `panels/`          | `core/panels/` + `ui/features/`                        |
| `render/`          | `ui/components/` + `ui/effects/`                       |
| `services/`        | `platform/` or future integration crates               |
| `settings/`        | `core/settings/` + `config/`                           |
| `taskbar/`         | `core/taskbar/` + `platform/windows/` + `ui/features/` |
| `meta/`            | `docs/references/`                                     |

Do not move the entire Windhawk `services/` directory into `platform/` without reviewing each service. Classify code by responsibility, not by its original folder name.

Examples:

- Weather API client → a future `integrations/weather` module.
- Brightness control → `platform/system/brightness.rs`.
- Media session access → `platform/media/session.rs`.
- Calendar state → `core/panels/calendar.rs`.
- Calendar rendering → `ui/features/calendar/`.

### 4.1 Deterministic Windhawk remodularization

Keep the scripts used to extract and merge Windhawk Mods separate from the Rust application source.

```text
scripts/
└── windhawk/
    ├── manifest.json
    ├── extract.ps1
    ├── merge.ps1
    ├── validate.ps1
    └── README.md

references/
└── windhawk/
    ├── original/
    ├── modularized/
    └── generated/
```

Apply the following rules:

1. `original/` stores the unmodified source references.
2. `modularized/` stores the reproducible, separated output.
3. `generated/` stores merge outputs and temporary artifacts.
4. Scripts must be deterministic: identical inputs must produce identical outputs.
5. Never manually edit generated files.

This keeps changes to the source references from silently corrupting or overwriting the Rust implementation.

## 5. Cargo workspace configuration

Define the workspace in the root `Cargo.toml`.

```toml
[workspace]
resolver = "2"
members = [
    "crates/app",
    "crates/core",
    "crates/platform",
    "crates/ui",
]

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

Example `crates/app/Cargo.toml`:

```toml
[package]
name = "nexora-app"
version.workspace = true
edition.workspace = true

[dependencies]
nexora-core = { path = "../core" }
nexora-platform = { path = "../platform" }
nexora-ui = { path = "../ui" }

tracing.workspace = true
tracing-subscriber.workspace = true
```

Internal dependencies:

| Crate      | Internal dependencies                      |
| ---------- | ------------------------------------------ |
| `app`      | `core`, `platform`, `ui`                   |
| `core`     | None                                       |
| `platform` | `core` (optional, for shared domain types) |
| `ui`       | `core`                                     |

Ideally, `core` has no dependency on other internal crates. If it requires an OS abstraction, define a trait or interface at the appropriate boundary rather than depending on a concrete platform implementation.

## 6. Rust development practices

### 6.1 Use a consistent module convention

For example:

```text
dock/
├── mod.rs
├── items.rs
├── layout.rs
└── visibility.rs
```

`mod.rs`:

```rust
pub mod items;
pub mod layout;
pub mod visibility;
```

Usage:

```rust
use crate::dock::items::DockItem;
```

Rust also supports file-based modules such as `dock.rs` alongside `dock/items.rs`. Both approaches are valid. Choose one convention and apply it consistently. This project uses `mod.rs` to make directory-level module boundaries explicit.

### 6.2 Avoid monolithic source files

Avoid putting the entire Dock implementation into a single `dock.rs` file that grows to thousands of lines.

Prefer splitting code by responsibility:

```text
dock/
├── mod.rs
├── state.rs
├── items.rs
├── layout.rs
├── behavior.rs
├── events.rs
└── visibility.rs
```

Do not over-fragment the codebase by placing every struct in its own file. File cohesion and responsibility boundaries matter more than file count.

### 6.3 Separate error handling

Use `thiserror` to define errors in library crates.

Example:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DockError {
    #[error("Dock item not found")]
    ItemNotFound,

    #[error("Invalid dock position")]
    InvalidPosition,
}
```

Use `tracing` for application logging. Avoid relying on `println!` for logging in a complex desktop application.

### 6.4 Use Cargo features deliberately

Cargo features can control which capabilities are enabled during a build.

Example:

```toml
[features]
default = ["dock", "dynamic-island"]

dock = []
dynamic-island = []
control-center = []
launchpad = []
```

Do not make every feature optional from the beginning. Introduce feature flags when distinct build configurations are actually required.

### 6.5 Separate configuration from runtime state

Keep persistent user preferences separate from transient application state.

For example:

- `DockSettings` — persistent user preferences.
- `DockState` — the Dock's current runtime state.

Do not combine these into a single type.

## 7. Potential crates for future growth

Add these crates only when the project reaches a point where their responsibilities justify separate boundaries.

| Crate          | When to introduce it                                                                        |
| -------------- | ------------------------------------------------------------------------------------------- |
| `integrations` | External API integrations become substantial, such as weather, lyrics, or calendar services |
| `storage`      | Persistence, databases, caching, and file management become more complex                    |
| `plugins`      | Third-party extensions need to be supported                                                 |
| `ipc`          | Multiple application processes need to communicate                                          |
| `updater`      | Automatic update functionality is implemented                                               |

A possible future process model:

```text
Nexora Desktop
│
├── Main Process
│   ├── Application lifecycle
│   ├── Settings
│   └── Feature orchestration
│
├── Dock Process
│   └── Dock rendering
│
├── Island Process
│   └── Dynamic Island rendering
│
└── Background Services
    ├── Media monitoring
    ├── Window monitoring
    └── System monitoring
```

This is a potential runtime decomposition, not a requirement to start with a multi-process architecture.

## 8. Development conventions

Run the following commands before submitting changes:

```powershell
# Formatting
cargo fmt --all

# Static analysis
cargo clippy --workspace --all-targets

# Run tests
cargo test --workspace

# Development build
cargo build --workspace

# Release build
cargo build --workspace --release

# Check without a full compilation
cargo check --workspace
```

Add GitHub Actions workflows:

```text
.github/
└── workflows/
    ├── ci.yml
    └── release.yml
```

At minimum, CI should run formatting checks, Clippy, and the test suite.

Document significant architectural choices using Architecture Decision Records (ADRs):

```text
docs/
└── decisions/
    ├── 0001-cargo-workspace.md
    ├── 0002-ui-rendering-framework.md
    ├── 0003-windows-api-strategy.md
    └── 0004-single-process-architecture.md
```

## 9. Implementation Order

Do not implement all features at once. Establish the foundations first, then build features on top of stable boundaries.

1. **Project foundation**
   - Set up the Cargo workspace and crate boundaries.
   - Establish logging, error handling, and configuration.
   - Set up the development and build pipeline.

2. **Windows platform foundation**
   - Implement window enumeration.
   - Retrieve process information.
   - Detect monitors and position windows.
   - Establish Windows Shell integration.

3. **UI foundation**
   - Implement the window host and rendering layer.
   - Establish reusable components, theming, and animation.
   - Implement transparency support.

4. **Dock implementation**
   - Implement Dock layout and positioning.
   - Support pinned and running applications.
   - Add tooltips, previews, and auto-hide behavior.

5. **Dynamic Island**
   - Implement Island state and event handling.
   - Add animations, media integration, timers, and notifications.

6. **Additional desktop features**
   - Implement Control Center, Launchpad, and Notification Center.
   - Add Hot Corners, Spotlight, and desktop effects.

7. **Polishing**
   - Build the settings UI and profile management.
   - Add localization and startup integration.
   - Implement crash recovery and performance optimization.
