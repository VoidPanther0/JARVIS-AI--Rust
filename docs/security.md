# Security Model

[`SECURITY.md`](../SECURITY.md) at the repository root covers how to
*report* a vulnerability. This document covers the actual design: what
JARVIS protects against, how, and where the current gaps are. See also
[`architecture.md`](architecture.md#permission-layer) for the permission
layer's place in the crate boundaries.

## Threat model

JARVIS runs locally with the user's own OS privileges and can, once a
feature requests it, read/write files and spawn processes on the user's
machine. The primary risks this project designs against:

1. **An AI provider's response causing unintended action.** Model output
   (local or cloud) is treated as untrusted input, same as any external
   data. It is never interpreted as a command to run.
2. **Arbitrary shell execution.** There is no "run this string" code path
   anywhere in the workspace. `SensitiveAction::RunProgram` takes a
   specific program name and a fixed argument vector, executed via
   `tokio::process::Command`, which does not invoke a shell — no shell
   metacharacter (`&&`, `|`, `$(...)`, etc.) is ever interpreted.
3. **Silent file/process access.** Every filesystem or process operation
   goes through `jarvis_platform::execute`, which requires a
   `PermissionDecision::Approved` obtained from an explicit user prompt.
   `jarvis-platform` is the *only* crate allowed to perform this kind of
   I/O; this is enforced by code review (`AGENTS.md` §5), not by a runtime
   sandbox.
4. **Leaked secrets.** No API keys, tokens, or credentials are committed.
   `.env` is git-ignored; `.env.example` documents the variable names only.
   Logs (`jarvis_core::logging`) are structured via `tracing` and nothing
   in the codebase logs an env var's value or file contents at `info`
   level or above.

## What is NOT yet enforced

Being explicit about this matters more than claiming a stronger posture
than the code has:

- **No OS-level sandboxing.** The permission layer is an in-process
  convention (a `SensitiveAction` must flow through `execute` after an
  `Approved` decision); it is not backed by an OS sandbox, seccomp
  profile, or AppContainer. A bug that bypassed the convention in code
  would not be caught by a second, independent enforcement layer. Until
  Phase 1 hardening explicitly adds one, treat the permission layer as a
  strong code-level contract, not a hard security boundary.
- **No signed installer yet.** The NSIS/MSI installers produced by the
  release workflow are not code-signed. Until a certificate is added (see
  [`release-process.md`](release-process.md#credentials-and-secrets)),
  Windows SmartScreen will show an unverified-publisher warning. This is
  expected, not a bug, at this stage.
- **`SensitiveAction::WriteFile` is a stub.** It validates a path but does
  not write bytes yet (see `docs/roadmap.md`), so it cannot currently be
  used to exfiltrate or corrupt data via content it never writes — but
  that also means it provides no real functionality to audit yet. Review
  this document again once it is implemented.
- **No rate limiting or cost controls on AI providers.** Once a real
  provider is wired up (Phase 2), calling it has no built-in guard against
  runaway spend/usage. This should be addressed as part of adding the
  first provider, not deferred further.

## Dependency hygiene

Dependencies are added only when an existing one does not already solve
the problem (`AGENTS.md` §8). `Cargo.lock` is committed so builds are
reproducible; dependency upgrades are reviewed individually rather than
bulk-updated.

## Reviewing a change for security impact

When reviewing or making a change, check whether it:

- Adds a new way to read, write, or execute something outside
  `jarvis-platform::execute`. If so, it needs its own `SensitiveAction`
  variant and permission-gated path, not a shortcut.
- Adds a dependency with filesystem, network, or process-spawning
  capability. If so, confirm it's actually needed and still minimal.
- Logs anything that could contain user content, file paths with personal
  information, or a credential. If so, remove it or drop it to a level
  that is off by default.
- Changes `apps/desktop/src-tauri/capabilities/*.json` or
  `tauri.conf.json`'s `security.csp`. These are the Tauri-level
  allow-lists for what the frontend can call and load; widen them only
  with a documented reason.
