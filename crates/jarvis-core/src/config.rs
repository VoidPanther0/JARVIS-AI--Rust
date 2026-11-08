//! Application configuration, loaded from environment variables and an
//! optional `.env` file. No secrets ship with the repository; see
//! `.env.example` for the variables a deployment may set.

use crate::error::{CoreError, CoreResult};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Logical runtime environment. Affects logging verbosity and defaults.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Production,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_defaults_to_development() {
        assert_eq!(Environment::default(), Environment::Development);
    }

    #[test]
    fn database_path_joins_fixed_filename() {
        let config = AppConfig {
            environment: Environment::Development,
            data_dir: PathBuf::from("/tmp/jarvis-data"),
            log_level: "info".to_string(),
            ai_provider: None,
        };
        assert_eq!(
            config.database_path(),
            PathBuf::from("/tmp/jarvis-data").join("jarvis.sqlite3")
        );
    }

    #[test]
    fn load_reads_environment_variables() {
        // All env mutation happens inside this single test function (and
        // is cleaned up at the end) so it can't race with other tests.
        // SAFETY: no other thread in this test binary reads/writes these
        // specific env vars concurrently.
        unsafe {
            std::env::set_var("JARVIS_ENV", "production");
            std::env::set_var("JARVIS_DATA_DIR", "/tmp/jarvis-test-data");
            std::env::set_var("JARVIS_LOG_LEVEL", "debug");
            std::env::set_var("JARVIS_AI_PROVIDER", "null");
        }

        let config = AppConfig::load().expect("load should succeed with valid env vars");

        assert_eq!(config.environment, Environment::Production);
        assert_eq!(config.data_dir, PathBuf::from("/tmp/jarvis-test-data"));
        assert_eq!(config.log_level, "debug");
        assert_eq!(config.ai_provider.as_deref(), Some("null"));

        // SAFETY: see above.
        unsafe {
            std::env::remove_var("JARVIS_ENV");
            std::env::remove_var("JARVIS_DATA_DIR");
            std::env::remove_var("JARVIS_LOG_LEVEL");
            std::env::remove_var("JARVIS_AI_PROVIDER");
        }
    }
}
