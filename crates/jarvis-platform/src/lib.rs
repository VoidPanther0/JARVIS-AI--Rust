//! Platform integration layer.
//!
//! The one place in the workspace allowed to spawn a process or touch
//! arbitrary files, and only through `execute`, which requires a prior
//! `PermissionDecision::Approved` from the caller (normally the desktop
//! UI, after showing the user what was requested and why). There is no
//! code path that runs a free-form shell string, and no AI response is
//! ever executed directly — it can only produce a `SensitiveAction` for
//! the user to approve or deny.

mod error;
mod executor;

pub use error::{PlatformError, PlatformResult};
pub use executor::execute;
