//! SQLite-backed persistent memory.
//!
//! Owns the connection pool, runs embedded migrations on startup, and
//! exposes a small, typed API over conversation history and key/value
//! personal memory. No AI logic lives here.

mod error;
mod store;

pub use error::{MemoryError, MemoryResult};
pub use store::{MemoryStore, Message, MessageRole};
