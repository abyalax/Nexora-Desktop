# Development Setup

This guide covers setting up the development environment for nexora, including Rust toolchain installation, dependencies, and available commands.

## Prerequisites

- **Windows 11** (build 22000 or newer), 64-bit
- **Administrator access** (for installing Visual Studio Build Tools)

## 1. Install Rust Toolchain

nexora uses Rust with the MSVC toolchain for Windows development.

### Install Rust

1. Download and run [rustup](https://rustup.rs) installer
2. During installation, select the **MSVC toolchain** for stable builds:
   ```powershell
   rustup default stable-x86_64-pc-windows-msvc
   ```
3. Verify installation:
   ```powershell
   rustc --version
   cargo --version
   ```

### Install Visual Studio Build Tools

Required for linking with Windows libraries:

1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022)
2. Run the installer and select **Desktop development with C++** workload
3. This includes:
   - MSVC linker (`link.exe`)
   - Windows 11 SDK
   - C++ build tools

> **Troubleshooting**: If you encounter "link.exe not found" errors, ensure the C++ workload is installed, or configure Cargo to use `rust-lld` by creating a local `.cargo/config.toml`.

### Install Git

Required for cloning the repository:

1. Download [Git for Windows](https://git-scm.com/download/win)
2. Install with default settings
3. Verify:
   ```powershell
   git --version
   ```

## 2. Clone the Repository

```powershell
git clone https://github.com/abyalax/Nexora-Desktop.git
cd nexora
```

## 4. Build the Project

### Debug Build

For development with debugging symbols:

```powershell
cargo build
```

The binaries will be in `target\debug\`:

- `nexora.exe` - Main application
- `nexora-guard.exe` - Watchdog process

### Release Build

For optimized production builds:

```powershell
cargo build --release
```

The binaries will be in `target\release\`:

- `nexora.exe` - Main application (stripped, optimized)
- `nexora-guard.exe` - Watchdog process

### Run the Application

After building:

```powershell
# Debug build
.\target\debug\nexora.exe

# Release build
.\target\release\nexora.exe
```

## 5. Available Commands

### Cargo Commands

#### Build Commands

```powershell
# Build all workspace crates (debug)
cargo build

# Build with optimizations (release)
cargo build --release

# Build specific crate
cargo build -p nexora-app
cargo build -p nexora-core
cargo build -p nexora-platform
cargo build -p nexora-ui
cargo build -p nacre-guard
```

#### Check & Lint

```powershell
# Check for compilation errors without building
cargo check

# Check specific crate
cargo check -p nexora-app

# Run Clippy linter
cargo clippy

# Run Clippy with all targets
cargo clippy --all-targets
```

#### Testing

```powershell
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p nexora-core

# Run tests with output
cargo test -- --nocapture

# Run tests in release mode
cargo test --release
```

#### Documentation

```powershell
# Generate and open documentation
cargo doc --open

# Generate documentation for all crates
cargo doc --workspace --open
```

#### Clean

```powershell
# Clean build artifacts
cargo clean

# Clean specific crate
cargo clean -p nexora-app
```

### Code Formatting

Rust code is formatted using `rustfmt` (part of the Rust toolchain):

```powershell
cargo fmt
cargo fmt --check
```

### Application CLI Flags

The `nexora.exe` supports several command-line flags for development and testing:

```powershell
# Keep Windows taskbar visible
.\target\release\nexora.exe --keep-taskbar

# Disable Deck
.\target\release\nexora.exe --no-dock

# Disable Visor
.\target\release\nexora.exe --no-island

# Disable HUD (volume/brightness overlay)
.\target\release\nexora.exe --no-hud

# Open Settings on startup
.\target\release\nexora.exe --open-settings

# Open Widgets on startup
.\target\release\nexora.exe --open-widgets

# Exit after N seconds (useful for testing)
.\target\release\nexora.exe --exit-after 30

# Add app to Deck
.\target\release\nexora.exe --add-app "C:\Path\To\App.exe"

# Set custom icon
.\target\release\nexora.exe --set-icon app "C:\Path\To\icon.png"
```

### Isolated Test Instance

Run nexora with a temporary config directory for safe testing:

```powershell
$env:NACRE_CONFIG_DIR = "$PWD\target\cfg-test"
.\target\release\nexora.exe --keep-taskbar --exit-after 30
```

This creates a temporary config in `target\cfg-test` instead of `%APPDATA%\nexora`.

## 6. Build Installer

To create the Windows installer and portable zip:

### Install Inno Setup

```powershell
winget install JRSoftware.InnoSetup
```

Or download from [Inno Setup website](https://jrsoftware.org/isinfo.php).

### Build Release

```powershell
# From the project root
powershell -File installer\build-installer.ps1 -Version 0.4.0
```

This script:

1. Builds nexora in release mode
2. Strips local paths and user names from binaries
3. Verifies no sensitive data remains in the binaries
4. Creates the installer: `dist\nexora-Setup-0.4.0.exe`
5. Creates the portable zip: `dist\nexora-0.4.0-win-x64.zip`

### Build Without Rebuilding

If you already have the binaries built:

```powershell
powershell -File installer\build-installer.ps1 -Version 0.4.0 -NoBuild
```

## 7. Project Structure

```
Desktop-Macs/
├── crates/
│   ├── app/          # Main application (nexora.exe)
│   ├── core/         # Business logic, settings, i18n
│   ├── platform/     # Windows-specific code (Win32, WinRT)
│   ├── ui/           # UI rendering (DirectComposition)
│   └── guard/        # Watchdog process (nexora-guard.exe)
├── assets/
│   ├── fonts/        # Bundled Inter font files
│   ├── screenshots/  # Documentation images
│   └── Nexora.ico      # Application icon
├── docs/
│   └── PANDUAN.md    # Indonesian user guide
├── installer/
│   ├── nexora.iss      # Inno Setup installer script
│   └── build-installer.ps1  # Build script
├── tools/            # Utility scripts
├── Cargo.toml        # Workspace configuration
├── package.json      # Prettier configuration
└── README.md         # User documentation
```

## 8. Development Workflow

### Typical Development Cycle

```powershell
# 1. Pull latest changes
git pull

# 2. Build in debug mode (faster compilation)
cargo build

# 3. Run the application
.\target\debug\nexora.exe

# 4. Make changes, rebuild
cargo build

# 5. Run tests
cargo test --workspace

# 6. Format code
cargo fmt
npx prettier --write .

# 7. Check with Clippy
cargo clippy --all-targets

# 8. Build release version when ready
cargo build --release
```

### Fast Development Build

For faster iteration, use the `fast` profile (less optimization, faster compilation):

```powershell
cargo build --profile fast
```

This uses the configuration from `Cargo.toml`:

```toml
[profile.fast]
inherits = "release"
lto = false
codegen-units = 16
```

## 9. Troubleshooting

### "link.exe not found"

**Solution**: Install Visual Studio Build Tools with the C++ workload, or configure Cargo to use rust-lld:

Create `.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"
ar = "llvm-ar"
```

### Font Issues

If text appears garbled or uses wrong fonts (e.g., Chinese fonts on Latin text):

1. Ensure `assets/fonts/` directory exists next to `nexora.exe`
2. Check logs for font loading status
3. The code uses `ANSI_CHARSET` to prevent Chinese font fallback

### Build Errors After Updates

```powershell
# Clean and rebuild
cargo clean
cargo build --release
```

### Rustup Update Issues

```powershell
# Update Rust toolchain
rustup update stable

# Update cargo
rustup self update
```

## 10. Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/) - Learn Rust
- [windows-rs Documentation](https://microsoft.github.io/windows-docs-rs/) - Windows API bindings
- [Windows.UI.Composition](https://docs.microsoft.com/en-us/uwp/api/windows.ui.composition) - Visual layer API
- [Inno Setup Documentation](https://jrsoftware.org/ishelp/) - Installer configuration

## 11. Contributing

Before contributing:

1. Format code with `cargo fmt` and `npx prettier --write .`
2. Run `cargo clippy --all-targets` and fix warnings
3. Run `cargo test --workspace` to ensure tests pass
4. Test with `--exit-after` flag to avoid messing up your real settings

See `README.md` for more information about the project.
