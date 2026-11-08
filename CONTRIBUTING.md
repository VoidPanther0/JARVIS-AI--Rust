# Contributing to JARVIS

## Development setup

See [`docs/development.md`](docs/development.md) for prerequisites and build
instructions.

## Workflow

1. Fork and branch from `main`.
2. Keep changes scoped: one crate or concern per pull request where possible.
3. Before opening a PR, run:
   ```
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```
4. Describe what changed and why in the PR body. Link related issues.

## Code organization

- `crates/jarvis-core` — configuration, logging, errors, permission model.
- `crates/jarvis-ai` — AI provider trait and registry (no provider committed).
- `crates/jarvis-voice` — voice I/O trait shape (not implemented yet).
- `crates/jarvis-memory` — SQLite storage and migrations.
- `crates/jarvis-platform` — permission-checked system access.
- `apps/desktop` — Tauri desktop application.

Shared logic belongs in a `crates/*` library, never duplicated per platform.
Anything that touches the filesystem or spawns a process outside of
`jarvis-platform::execute` (behind an explicit permission check) will be
rejected in review.

## Commit style

Use clear, imperative commit messages (`Add SQLite migration runner`, not
`fixed stuff`). Squash fixup commits before requesting review.

## Reporting bugs / proposing features

Open a GitHub issue with reproduction steps (for bugs) or motivation and
scope (for features).
