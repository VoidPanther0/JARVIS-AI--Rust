//! Provider-agnostic AI abstraction.
//!
//! No concrete AI backend (OpenAI, local llama.cpp, Anthropic, etc.) is
//! wired up in Phase 1. This crate only defines the trait every future
//! provider must implement and a registry to look one up by name, plus a
//! `NullProvider` used so the app builds and runs end-to-end before any
//! real provider is added.

mod error;
mod null_provider;
mod provider;
mod registry;

pub use error::{AiError, AiResult};
pub use null_provider::NullProvider;
pub use provider::{AiProvider, ChatMessage, ChatRequest, ChatResponse, Role};
pub use registry::ProviderRegistry;
