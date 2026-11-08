//! Provider-agnostic AI abstraction.
//!
//! `NullProvider` is the always-available placeholder; `OpenRouterProvider`
//! and `NvidiaProvider` (in `providers::`) are the two real cloud
//! backends. [`orchestrator::Orchestrator`] is the entry point that ties
//! key rotation ([`keys::KeyPool`]), the model catalog ([`models`]), and
//! task-based model selection ([`selection`]) together — see its module
//! docs for the retry/fallback policy.

mod error;
pub mod keys;
pub mod models;
mod null_provider;
pub mod orchestrator;
mod provider;
pub mod providers;
mod registry;
pub mod selection;
pub mod settings;

pub use error::{AiError, AiResult};
pub use null_provider::NullProvider;
pub use orchestrator::Orchestrator;
pub use provider::{AiProvider, ChatMessage, ChatRequest, ChatResponse, Role};
pub use registry::ProviderRegistry;
