# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added
- Unit tests for every crate (`jarvis-core`, `jarvis-ai`, `jarvis-memory`,
  `jarvis-platform`, `jarvis-voice`): configuration loading, the
  permission-request vocabulary, the provider registry and `NullProvider`,
  SQLite conversation/key-value round-trips, permission-gated file
  read/delete, and the voice trait default behavior. `cargo test --workspace`
  previously ran 0 tests.
- `docs/architecture.md`, `docs/roadmap.md`, `docs/release-process.md`, and
  `docs/security.md`.

### Fixed
- `Cargo.toml`'s `repository` field pointed at a placeholder
  (`jarvis-ai/jarvis`) instead of the actual GitHub repository.
- `AppState::memory()` (desktop app) panicked (`.expect`) if the SQLite
  store failed to open after startup; it now returns a `Result` that the
  `send_message` command propagates as an error instead of crashing the
  app.
- Renamed `Readme.md` to `README.md` for GitHub convention.

### Changed
- `.github/workflows/ci.yml` now declares an explicit
  `permissions: contents: read` at the workflow level (least privilege).

## [0.1.0-alpha.1] - 2026-11-08

### Added
- Rust workspace scaffolding: `jarvis-core`, `jarvis-ai`, `jarvis-voice`,
  `jarvis-memory`, `jarvis-platform`, and the `apps/desktop` Tauri app.
- Configuration loading from environment variables and `.env`.
- Structured logging via `tracing`.
- SQLite storage with embedded migrations (`jarvis-memory`).
- Provider-agnostic AI abstraction with a placeholder `NullProvider`
  (`jarvis-ai`); no concrete AI backend is wired up yet.
- Permission-checked command execution model (`jarvis-core::permissions`,
  `jarvis-platform::execute`); no arbitrary shell execution.
- Minimal desktop chat UI (HTML/CSS/JS) backed by a single Tauri command.
- GitHub Actions: format/clippy/test checks, Windows build, NSIS/MSI
  installer generation, and release publishing on `v*` tags.
- MIT license, README, CONTRIBUTING, SECURITY, and this changelog.

### Not implemented yet
- Voice input/output (`jarvis-voice` only defines the trait shape).
- Any concrete AI provider.
- Wake-word activation, desktop automation beyond the permission model,
  Linux/Android/macOS/iOS packaging.
