//! Shared foundation for JARVIS: configuration, logging, errors, and
//! the permission model used before any sensitive operation runs.

pub mod config;
pub mod error;
pub mod logging;
pub mod permissions;
pub mod secrets;

pub use config::AppConfig;
pub use error::{CoreError, CoreResult};
pub use permissions::{PermissionDecision, PermissionRequest, SensitiveAction};
