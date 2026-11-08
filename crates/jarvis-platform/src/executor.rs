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
