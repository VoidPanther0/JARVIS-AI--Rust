# Security Policy

JARVIS runs on the user's own machine with access to their files and
processes, so security issues are treated as high priority.

## Reporting a vulnerability

Please do **not** open a public GitHub issue for security vulnerabilities.
Instead, open a private security advisory via GitHub's "Report a
vulnerability" feature on this repository, or contact the maintainers
directly through a channel listed on the repository's GitHub profile.

Include:
- A description of the issue and its impact.
- Steps to reproduce.
- Affected version/commit.

## Design principles this project follows

- No arbitrary shell command execution. `jarvis-platform` only runs a
  pre-validated program with a fixed argument list.
- Every sensitive action (running a program, reading/writing/deleting a
  file) is modeled as a `SensitiveAction` and requires an explicit
  `PermissionDecision::Approved` obtained from the user before it runs.
- No API keys or secrets are committed to the repository. Configuration is
  read from environment variables; see `.env.example`.
- Dependencies are kept minimal and reviewed before being added.

## Supported versions

This project is in Phase 1 (`0.1.0-alpha.x`). Only the latest released
version receives fixes during this stage.
