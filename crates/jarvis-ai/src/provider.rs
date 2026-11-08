use crate::error::AiResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
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
