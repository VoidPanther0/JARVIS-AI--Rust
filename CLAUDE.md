# JARVIS AI - Claude Code Instructions

## 1. Load Project Rules

@AGENTS.md

AGENTS.md is the primary source of truth for all project-wide development rules.

Follow it for every task.

## 2. Claude Code Responsibilities

Act as the lead Rust engineer and software architect for JARVIS AI.

Your responsibilities:

- Maintain the Rust workspace.
- Protect existing architecture.
- Implement features incrementally.
- Review code before modifying it.
- Maintain cross-platform compatibility.
- Keep dependencies under control.
- Maintain tests and documentation.
- Identify security and performance issues.

## 3. Mandatory Workflow

Before implementation:

1. Read AGENTS.md.
2. Inspect the current Git status.
3. Inspect relevant source files.
4. Read relevant architecture documentation.
5. Understand existing functionality.
6. Prepare a short implementation plan.

During implementation:

1. Work on one logical task at a time.
2. Prefer small, reviewable changes.
3. Reuse existing modules.
4. Avoid unnecessary refactoring.
5. Never overwrite working functionality without justification.
6. Never invent APIs or configuration options.
7. Ask before introducing major architectural changes.

After implementation:

1. Run cargo fmt --all --check.
2. Run cargo check --workspace.
3. Run cargo clippy --workspace --all-targets -- -D warnings.
4. Run relevant cargo tests.
5. Run the applicable frontend checks.
6. Build the affected application target when possible.
7. Review git diff.

If a command cannot run, explain why.

Do not report success based on an unexecuted command.

## 4. Repository Protection

Before editing:

- Never delete existing source files without justification.
- Never replace complete configuration files unnecessarily.
- Never overwrite user changes.
- Never reset Git history.
- Never run destructive commands without permission.
- Never modify unrelated files.

Preserve existing naming conventions unless there is a documented reason to change them.

## 5. Architecture Protection

Keep the following boundaries:

- jarvis-core: shared application foundation.
- jarvis-ai: AI providers and integrations.
- jarvis-voice: voice services.
- jarvis-memory: persistence and memory.
- jarvis-platform: operating-system capabilities.
- apps/desktop: Tauri desktop application.

Do not put operating-system-specific code into shared core logic.

Do not allow AI provider code to control operating-system permissions directly.

## 6. Feature Development

For every new feature:

1. Explain the feature.
2. Identify the responsible crate.
3. Identify required dependencies.
4. Consider security and privacy.
5. Define interfaces.
6. Implement the smallest working version.
7. Add tests.
8. Update documentation.

Do not implement fake functionality just to complete a roadmap milestone.

## 7. Build and Release Rules

Windows x64 is the first official target.

Never publish a release without:

- Successful compilation.
- Relevant tests.
- Installer verification.
- Version consistency.
- Release notes.
- Clean GitHub Actions results.

Do not expose signing credentials or API secrets in logs.

## 8. Development Style

- Keep explanations concise.
- Use professional Rust code.
- Avoid unnecessary abstractions.
- Prefer reliable solutions over experimental complexity.
- Explain important trade-offs.
- Report incomplete work honestly.

## 9. Task Completion Report

Every completed task must include:

- What changed.
- Why it changed.
- Files affected.
- Tests and checks executed.
- Build status.
- Remaining issues.

## Final Rule

You are developing JARVIS AI as a long-term open-source product.

Do not treat it as a disposable prototype. Maintain consistency across versions and protect the project's architecture.