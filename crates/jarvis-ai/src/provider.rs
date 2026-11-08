use crate::error::AiResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    /// Model to use for this request; `None` lets the provider fall back
    /// to its configured default.
    #[serde(default)]
    pub model: Option<String>,
    /// The resolved API key for this call, rotated in by the caller's key
    /// pool. `None` for providers that don't need one (e.g. `NullProvider`).
    /// Never serialized and never shown in `Debug` output — see
    /// `jarvis_core::secrets::mask` for displaying a key elsewhere.
    #[serde(default, skip_serializing)]
    pub api_key: Option<String>,
}

impl fmt::Debug for ChatRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ChatRequest")
            .field("messages", &self.messages)
            .field("model", &self.model)
            .field("api_key", &self.api_key.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: ChatMessage,
}

/// Implemented by every AI backend JARVIS can talk to. No provider is
/// committed to in Phase 1; see `NullProvider` for the placeholder used
/// until a real one (local or cloud) is added.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Stable identifier used for configuration and registry lookup.
    fn name(&self) -> &str;

    async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse>;
}
