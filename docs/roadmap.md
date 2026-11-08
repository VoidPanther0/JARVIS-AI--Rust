# Roadmap

This reflects the actual state of the code, not aspirational milestones.
See [`CHANGELOG.md`](../CHANGELOG.md) for the authoritative, dated record of
what has shipped. Platform order follows [`AGENTS.md`](../AGENTS.md#2-development-priorities).

## Phase 1 — Foundation (current, `0.1.0-alpha.x`)

Goal: a working, honest skeleton — every crate boundary exists, the
desktop app builds and runs on Windows, and no part of the system fakes a
capability it doesn't have.

Status:

| Area | State |
|---|---|
| Workspace scaffolding (`jarvis-core`, `jarvis-ai`, `jarvis-voice`, `jarvis-memory`, `jarvis-platform`, `apps/desktop`) | Implemented |
| Configuration (env vars + `.env`) | Implemented |
| Structured logging | Implemented |
| SQLite storage + migrations | Implemented |
| Permission model (`SensitiveAction` / approval flow) | Implemented |
| Permission-checked process/file execution | Implemented (`RunProgram`, `ReadFile`, `DeleteFile`; `WriteFile` validates the path only — see below) |
| AI provider abstraction + registry | Implemented |
| Concrete AI provider (local or cloud) | **Not implemented** — only `NullProvider` exists |
| Minimal desktop chat UI | Implemented |
| Voice input/output | **Not implemented** — `jarvis-voice` only defines trait shapes |
| Wake-word detection | **Not implemented** |
| Windows CI (fmt/clippy/test) + installer build (NSIS/MSI) | Implemented |
| Unit test coverage per crate | Implemented (baseline coverage; expand per feature) |

Known gap carried into Phase 1 follow-up: `SensitiveAction::WriteFile` is
accepted by the permission model but `jarvis_platform::execute` does not
yet write bytes to disk for it (see `executor.rs`) — it only confirms the
path. Closing this requires deciding how content is supplied to `execute`
without widening what counts as a "pre-validated" write.

## Phase 2 — First real AI provider

- Implement one concrete `AiProvider` (local, CPU-only model or a cloud API
  behind an explicit, user-supplied key — never hardcoded).
- Wire `SensitiveAction::WriteFile` to actually write content.
- Expand `jarvis-memory` usage: surface conversation history in the UI
  instead of only persisting it.

## Phase 3 — Voice

- Implement `SpeechToText` against a Whisper-compatible backend.
- Implement `TextToSpeech` against a Piper-compatible backend.
- Pluggable wake-word detection, gated behind an explicit opt-in (always
  listening is a sensitive default and must be presented as one).

## Phase 4 — Additional platforms

In priority order (per `AGENTS.md`): Linux desktop, Android, macOS, iOS.
Each platform gets its own adapter in `jarvis-platform` (or a sibling
crate, if OS APIs diverge enough to warrant it) — shared crates stay
platform-independent throughout.

## Out of scope until explicitly planned

- Arbitrary shell command execution (permanently out of scope — see
  [`SECURITY.md`](../SECURITY.md)).
- Plugin/extension system.
- Multi-user support.

## Versioning note

The project is in `0.x` and uses `-alpha.n` / `-beta.n` prerelease
identifiers (see `AGENTS.md` §12). A version bump happens when a phase
above actually ships, not on a calendar — this roadmap does not promise
dates.
