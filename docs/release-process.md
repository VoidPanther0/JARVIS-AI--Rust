# Release Process

Windows x64 is the only officially released target right now (see
[`AGENTS.md`](../AGENTS.md#2-development-priorities)). This document
describes the actual pipeline in `.github/workflows/`, not an aspirational
one.

## Before cutting a release

Every item below must be true — per `AGENTS.md` §7, never publish a release
from an unverified build:

- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes.
- [ ] `cargo test --workspace` passes.
- [ ] The desktop app builds (`npm run build` in `apps/desktop`) and the
      resulting NSIS `.exe` and `.msi` installers have been smoke-tested
      (install, launch, uninstall) on Windows 10/11.
- [ ] `CHANGELOG.md` has an entry for the release under a dated, versioned
      heading (not just "Unreleased").
- [ ] The version in `Cargo.toml` (`workspace.package.version`),
      `apps/desktop/package.json`, and
      `apps/desktop/src-tauri/tauri.conf.json` agree with each other and
      with the changelog entry and the git tag about to be pushed.
- [ ] CI is green on the commit being tagged.

## Versioning

Semantic Versioning, per `AGENTS.md` §12:

- Development: `0.x.y-alpha.n`, `0.x.y-beta.n`.
- Stable: `1.0.0`, `1.0.1`, `1.1.0`, `2.0.0`.

Never mark a release stable (`1.x` or later) without the full checklist
above *and* installer verification — a passing test suite alone is not
sufficient per the project's definition of done (`AGENTS.md` §15).

## Cutting a release

1. Update the three version locations listed above to the same value.
2. Add/finalize the `CHANGELOG.md` entry for that version.
3. Commit (`build: prepare vX.Y.Z-suffix`), push, and confirm CI
   (`.github/workflows/ci.yml`) is green on `main`.
4. Tag the commit `vX.Y.Z-suffix` and push the tag. This is the only
   trigger for `.github/workflows/release.yml`.

```powershell
git tag vX.Y.Z-suffix
git push origin vX.Y.Z-suffix
```

## What the release workflow does

`.github/workflows/release.yml`, triggered on `v*` tags:

1. Checks out the tag, installs the Rust toolchain and Node.js.
2. Runs `cargo test --workspace` — the workflow stops here if tests fail.
3. Builds the Tauri app (`npm run build`), producing NSIS and MSI
   installers under `apps/desktop/src-tauri/target/release/bundle/`.
4. Publishes a **draft** GitHub Release (`softprops/action-gh-release`,
   `draft: true`) with the installers attached and auto-generated release
   notes.

The job runs with `permissions: contents: write`, scoped to this job only,
to create the release and upload assets.

## After the workflow finishes

The release is a **draft** — nothing is public yet. A maintainer must:

1. Open the draft release on GitHub.
2. Verify both installer assets are present and correctly named.
3. Download and verify at least one installer actually installs and runs.
4. Edit the auto-generated notes if needed (they come from commit
   messages/PR titles, not hand-written).
5. Publish the release manually.

This project does not auto-publish releases — that step is always a human
action, and no automation in this repository is authorized to do it.

## Credentials and secrets

The release workflow only uses the repository's built-in `GITHUB_TOKEN`
(implicitly, via `softprops/action-gh-release`), scoped by the explicit
`permissions:` block. No other secrets are required for the current
Windows-only pipeline. If a future signing step (code-signing certificate)
is added, the certificate/key must be stored as a GitHub Actions secret,
never committed, and never printed to logs — see
[`SECURITY.md`](../SECURITY.md).
