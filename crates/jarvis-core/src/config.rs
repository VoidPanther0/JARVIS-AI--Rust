//! Application configuration, loaded from environment variables and an
//! optional `.env` file. No secrets ship with the repository; see
//! `.env.example` for the variables a deployment may set.

use crate::error::{CoreError, CoreResult};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logical runtime environment. Affects logging verbosity and defaults.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub environment: Environment,
    /// Directory holding the SQLite database and other persistent state.
    pub data_dir: PathBuf,
    /// Minimum tracing level, e.g. "info", "debug".
    pub log_level: String,
    /// Name of the AI provider to use; resolved by `jarvis-ai` at runtime.
    pub ai_provider: Option<String>,
}

impl AppConfig {
    /// Loads configuration from environment variables, falling back to
    /// per-OS application data directories for anything unset.
    pub fn load() -> CoreResult<Self> {
        let dirs = ProjectDirs::from("ai", "JARVIS", "JARVIS")
            .ok_or_else(|| CoreError::Config("could not resolve application directories".into()))?;

        let environment = match std::env::var("JARVIS_ENV").as_deref() {
            Ok("production") => Environment::Production,
            _ => Environment::Development,
        };

        let data_dir = std::env::var("JARVIS_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs.data_dir().to_path_buf());

        let log_level = std::env::var("JARVIS_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
        let ai_provider = std::env::var("JARVIS_AI_PROVIDER").ok();

        Ok(Self {
            environment,
            data_dir,
            log_level,
            ai_provider,
        })
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("jarvis.sqlite3")
    }
}
