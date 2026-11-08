# Architecture

This document describes how the JARVIS workspace is split into crates, why
the boundaries are drawn where they are, and how data flows through the
desktop application. For setup and local build steps, see
[`development.md`](development.md). For the release pipeline, see
[`release-process.md`](release-process.md).

## Workspace layout

```
apps/
  desktop/
    src-tauri/   Tauri shell: window, commands, app state
    ui/          HTML/CSS/JS frontend (no framework)
crates/
  jarvis-core/      config, errors, logging, permission model
  jarvis-ai/        AI provider trait + registry
  jarvis-voice/     speech-to-text / text-to-speech trait shape
  jarvis-memory/    SQLite storage + migrations
  jarvis-platform/  permission-checked process/filesystem execution
```

`Cargo.toml` at the repository root defines the workspace and the shared
dependency versions every crate pulls from (`workspace.dependencies`), so a
library version is only ever declared in one place.

## Crate responsibilities

### `jarvis-core`
The only crate every other crate is allowed to depend on. It owns:

- `config` — `AppConfig::load()` reads environment variables (optionally via
  a `.env` file) and resolves an OS-appropriate data directory via
  `directories::ProjectDirs`. Nothing else in the workspace reads
  `std::env` directly for application configuration.
- `error` — `CoreError`/`CoreResult`, the error type other core-level
  concerns use.
- `logging` — one `tracing_subscriber` initialization function, called
  once by the desktop binary at startup.
- `permissions` — the `SensitiveAction` / `PermissionRequest` /
  `PermissionDecision` vocabulary described below. This module defines the
  *types*; it does not execute anything.

### `jarvis-ai`
Defines `AiProvider` (an `async_trait`) and `ProviderRegistry`, a name →
provider lookup. Ships one implementation, `NullProvider`, which returns an
explanatory message instead of pretending to be a real model. Adding a real
backend (local or cloud) means implementing `AiProvider` and registering it;
it does not require changing the registry, the desktop command, or the UI.

No AI-provider code may reach into `jarvis-platform` or request OS
permissions directly — see [Permission layer](#permission-layer) below.

### `jarvis-voice`
Defines `SpeechToText` and `TextToSpeech` traits with default methods that
return `VoiceError::NotImplemented`. No STT/TTS backend is wired up yet;
this crate exists so the eventual Whisper-compatible and Piper-compatible
integrations have a stable shape to implement against from day one, and so
other crates can depend on the trait without depending on a concrete engine.

### `jarvis-memory`
Owns the single `sqlx::SqlitePool` and the embedded migration set
(`src/migrations/*.sql`, run automatically by `MemoryStore::open`). Exposes
a typed API (`start_conversation`, `add_message`, `set_memory`,
`get_memory`) — callers never write raw SQL outside this crate. All I/O is
async and goes through `tokio`; there are no blocking filesystem/database
calls on the async runtime.

### `jarvis-platform`
The only crate allowed to spawn a process or touch a file outside the
database path. Its single public entry point, `execute`, requires a
`PermissionDecision` the caller already obtained from the user — this crate
never prompts anyone itself, and there is no code path that interprets a
string as a shell command (`RunProgram` takes a fixed program name and
argument list, run via `tokio::process::Command`, never a shell).

### `apps/desktop`
The Tauri application: `main.rs` (entry point), `lib.rs` (wires config,
logging, and the Tauri builder), `state.rs` (`AppState`, holding the config,
the AI provider registry, and a lazily-opened `MemoryStore`), and
`commands.rs` (the `#[tauri::command]` functions the frontend calls via
`invoke`). The frontend (`apps/desktop/ui`) is plain HTML/CSS/JS with no
build step beyond what the Tauri CLI already runs.

## Permission layer

JARVIS treats anything that touches the user's files or processes as a
`SensitiveAction` (`jarvis_core::permissions`):

```
SensitiveAction::RunProgram { program, args }
SensitiveAction::ReadFile { path }
SensitiveAction::WriteFile { path }
SensitiveAction::DeleteFile { path }
```

The flow is always: something wants to perform an action → it constructs a
`PermissionRequest` describing the action and the reason → the UI shows
this to the user and returns a `PermissionDecision` → only then can
`jarvis_platform::execute(action, decision)` run it, and it refuses
immediately if the decision isn't `Approved`. An AI provider's output is
treated as untrusted: it can *describe* a desired action, but the only way
that action actually runs is through this same approval path — no response
is ever executed directly.

## Data flow: a chat message

1. The UI (`ui/main.js`) calls `invoke("send_message", { message })`.
2. `commands::send_message` (desktop `src-tauri`) looks up the configured
   provider in `AppState.providers` (`jarvis-ai`), sends the message, and
   gets a reply.
3. Both sides of the exchange are persisted through
   `AppState::memory()` → `jarvis-memory::MemoryStore`.
4. The reply string is returned to the frontend, which appends it to the
   chat log.

No step in this path touches `jarvis-platform`; chat has no filesystem or
process access. That only happens once a feature explicitly constructs a
`SensitiveAction` and the permission flow above is run.

## Cross-platform boundaries

Only `jarvis-platform` and `apps/desktop` are allowed to contain
OS-specific code (`cfg(windows)`, `cfg(target_os = "linux")`, etc.).
`jarvis-core`, `jarvis-ai`, `jarvis-voice`, and `jarvis-memory` must stay
portable — none of them assume a desktop filesystem, a microphone, or a
process-control API exists, so they can eventually run on Android without
being rewritten.
