# Local Development

## Prerequisites (Windows 10/11)

1. **Rust** (stable) via [rustup](https://rustup.rs).
   ```powershell
   winget install Rustlang.Rustup
   rustup default stable
   ```
2. **Node.js** 18+ (for the Tauri CLI) — https://nodejs.org.
3. **Microsoft C++ Build Tools** (Desktop development with C++ workload) —
   required by Tauri on Windows. Install via Visual Studio Installer or
   `winget install Microsoft.VisualStudio.2022.BuildTools`.
4. **WebView2 Runtime** — preinstalled on Windows 11; on Windows 10 install
   from Microsoft if missing.
5. **Tauri CLI**, installed per-project via npm (see below) rather than
   globally, to keep versions pinned.

No GPU is required; everything in Phase 1 runs on CPU.

## First-time setup

```powershell
cd "apps/desktop"
npm install
```

## Running in development

```powershell
cd "apps/desktop"
npm run dev
```

This launches the Tauri dev window with the UI in `apps/desktop/ui` and the
Rust backend in `apps/desktop/src-tauri`, rebuilding on change.

Copy `.env.example` to `.env` at the repository root first if you want to
override defaults (data directory, log level, AI provider name).

## Workspace checks

Run these from the repository root before committing:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Building the Windows installer

Tauri's icon files must exist before a release build; generate them once
from a source image (1024x1024 PNG recommended):

```powershell
cd "apps/desktop"
npx tauri icon path\to\source-icon.png
```

Then build:

```powershell
cd "apps/desktop"
npm run build
```

This produces both an NSIS `.exe` installer and an `.msi` installer under
`apps/desktop/src-tauri/target/release/bundle/`.

## Project layout

```
apps/desktop/        Tauri application (Rust backend + HTML/CSS/JS frontend)
crates/jarvis-core/   Config, logging, errors, permission model
crates/jarvis-ai/     AI provider trait + registry (no provider committed)
crates/jarvis-voice/  Voice I/O trait shape (not implemented)
crates/jarvis-memory/ SQLite storage + migrations
crates/jarvis-platform/ Permission-checked system access
```
