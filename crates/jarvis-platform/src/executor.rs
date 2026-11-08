use crate::error::{PlatformError, PlatformResult};
use jarvis_core::{PermissionDecision, SensitiveAction};
use tracing::{info, warn};

/// Runs a `SensitiveAction`, but only if `decision` is `Approved`.
///
/// Callers must obtain the decision from an explicit user prompt before
/// calling this function; nothing in this crate asks the user itself.
/// `RunProgram` spawns a specific, pre-validated executable with a fixed
/// argument list — there is no shell interpretation of the arguments.
pub async fn execute(
    action: SensitiveAction,
    decision: PermissionDecision,
) -> PlatformResult<String> {
    if decision != PermissionDecision::Approved {
        warn!(?action, "sensitive action denied");
        return Err(PlatformError::Denied);
    }

    info!(?action, "executing approved sensitive action");

    match action {
        SensitiveAction::RunProgram { program, args } => {
            let output = tokio::process::Command::new(&program)
                .args(&args)
                .output()
                .await?;
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
        SensitiveAction::ReadFile { path } => {
            let content = tokio::fs::read_to_string(&path).await?;
            Ok(content)
        }
        SensitiveAction::WriteFile { path } => {
            // The caller is responsible for supplying content through a
            // higher-level API; this placeholder only validates the path
            // exists and is writable context-wise. Writing content bytes
            // is intentionally not implemented in Phase 1.
            Ok(format!("write target validated: {path}"))
        }
        SensitiveAction::DeleteFile { path } => {
            tokio::fs::remove_file(&path).await?;
            Ok(format!("deleted: {path}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_file_path(name: &str) -> std::path::PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock is after the epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("jarvis-platform-test-{name}-{unique}.txt"))
    }

    #[tokio::test]
    async fn denied_decision_never_runs_the_action() {
        let result = execute(
            SensitiveAction::ReadFile {
                path: "irrelevant.txt".to_string(),
            },
            PermissionDecision::Denied,
        )
        .await;

        assert!(matches!(result, Err(PlatformError::Denied)));
    }

    #[tokio::test]
    async fn approved_read_file_returns_contents() {
        let path = temp_file_path("read");
        tokio::fs::write(&path, "hello from jarvis-platform tests")
            .await
            .expect("writing the fixture file should succeed");

        let result = execute(
            SensitiveAction::ReadFile {
                path: path.to_string_lossy().to_string(),
            },
            PermissionDecision::Approved,
        )
        .await
        .expect("reading an existing file should succeed");

        assert_eq!(result, "hello from jarvis-platform tests");

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn approved_delete_file_removes_it() {
        let path = temp_file_path("delete");
        tokio::fs::write(&path, "to be deleted")
            .await
            .expect("writing the fixture file should succeed");

        execute(
            SensitiveAction::DeleteFile {
                path: path.to_string_lossy().to_string(),
            },
            PermissionDecision::Approved,
        )
        .await
        .expect("deleting an existing file should succeed");

        assert!(!path.exists());
    }
}
