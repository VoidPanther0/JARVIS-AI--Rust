# JARVIS AI

Open-source personal AI assistant built with Rust.

JARVIS is a cross-platform AI assistant designed to bring voice interaction, intelligent conversations, and desktop automation to Windows, Linux, macOS, Android, and eventually iOS.

**Status: Phase 1 (foundation).** See [CHANGELOG.md](CHANGELOG.md) for what
is implemented so far and what is intentionally still a stub.

## Planned Features
- AI-powered conversations
- Voice recognition and speech synthesis
- Wake-word activation
- Desktop and system automation
- Personal memory
- Cross-platform interface
- Plugin and extension support

## Supported Platforms

| Platform | Status |
|---|---|
| Windows 10/11 | First target |
| Linux | Planned |
| Android | Planned |
| macOS | Planned |
| iOS | Future |

## Technology
- Rust
- Tauri 2
- SQLite (via `sqlx`)
- Tokio
- Whisper / Piper (planned for voice, not yet wired up)

## Project structure

```
apps/
  desktop/              Tauri desktop application
    src-tauri/          Rust backend (commands, state, Tauri config)
    ui/                 HTML/CSS/JS frontend
crates/
  jarvis-core/          Config, logging, errors, permission model
  jarvis-ai/            AI provider trait + registry (no provider committed)
  jarvis-voice/         Voice I/O trait shape (not implemented yet)
  jarvis-memory/        SQLite storage + migrations
  jarvis-platform/      Permission-checked system access
docs/                   Development and architecture docs
.github/workflows/      CI and release automation
```

## Installation

Official installers will be published through GitHub Releases when available.

## Development

See [docs/development.md](docs/development.md) for prerequisites, running
the app locally, and building the Windows installer.

```powershell
cd apps/desktop
npm install
npm run dev
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md) for the project's safety model and how to
report vulnerabilities.

## License

MIT License — see [LICENSE](LICENSE).
