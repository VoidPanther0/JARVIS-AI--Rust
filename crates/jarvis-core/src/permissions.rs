//! Permission model for sensitive operations.
//!
//! JARVIS never executes arbitrary shell commands or touches the filesystem
//! outside its own data directory without an explicit, structured request
//! that the user interface presents to the user for approval. This module
//! defines the vocabulary; the desktop app's Tauri commands are the only
//! place a `SensitiveAction` is turned into real work, and only after a
//! `PermissionDecision::Approved` has been returned by the user.

use serde::{Deserialize, Serialize};

/// A categorized action that touches the user's system and therefore
/// requires explicit, per-call approval before it runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SensitiveAction {
    /// Run a specific, pre-validated executable with fixed arguments.
    /// There is no "run this string as a shell command" variant.
    RunProgram {
        program: String,
        args: Vec<String>,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
    },
    DeleteFile {
        path: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRequest {
    pub action: SensitiveAction,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionDecision {
    Approved,
    Denied,
}

impl PermissionRequest {
    pub fn new(action: SensitiveAction, reason: impl Into<String>) -> Self {
        Self {
            action,
            reason: reason.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_action_and_reason() {
        let request = PermissionRequest::new(
            SensitiveAction::ReadFile {
                path: "C:/data/notes.txt".to_string(),
            },
            "user asked to summarize this file",
        );

        assert_eq!(request.reason, "user asked to summarize this file");
        match request.action {
            SensitiveAction::ReadFile { path } => assert_eq!(path, "C:/data/notes.txt"),
            other => panic!("expected ReadFile, got {other:?}"),
        }
    }

    #[test]
    fn sensitive_action_round_trips_through_json() {
        let action = SensitiveAction::RunProgram {
            program: "git".to_string(),
            args: vec!["status".to_string()],
        };

        let json = serde_json::to_string(&action).expect("serialize");
        let decoded: SensitiveAction = serde_json::from_str(&json).expect("deserialize");

        match decoded {
            SensitiveAction::RunProgram { program, args } => {
                assert_eq!(program, "git");
                assert_eq!(args, vec!["status".to_string()]);
            }
            other => panic!("expected RunProgram, got {other:?}"),
        }
    }

    #[test]
    fn permission_decision_distinguishes_approved_and_denied() {
        assert_ne!(PermissionDecision::Approved, PermissionDecision::Denied);
    }
}
