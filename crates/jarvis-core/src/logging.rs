//! Structured logging setup shared by every JARVIS binary.

use tracing_subscriber::{fmt, EnvFilter};

/// Initializes a global tracing subscriber. Call once, at process start.
///
/// `default_level` is used when `RUST_LOG` / `JARVIS_LOG_LEVEL` is unset.
pub fn init(default_level: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    fmt()
        .with_env_filter(filter)
        .with_target(true)
        .with_level(true)
        .init();
}
