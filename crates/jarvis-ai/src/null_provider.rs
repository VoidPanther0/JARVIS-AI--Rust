use crate::error::AiResult;
use crate::provider::{AiProvider, ChatMessage, ChatRequest, ChatResponse, Role};
use async_trait::async_trait;

/// Placeholder provider that explains itself rather than pretending to
/// think. Used as the default until a real provider is configured, so the
/// app has honest, working behavior instead of a hidden stub.
pub struct NullProvider;

#[async_trait]
impl AiProvider for NullProvider {
    fn name(&self) -> &str {
        "null"
    }

    async fn chat(&self, _request: ChatRequest) -> AiResult<ChatResponse> {
        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: "No AI provider is configured yet. Set JARVIS_AI_PROVIDER and wire up \
                          a provider implementing `jarvis_ai::AiProvider` to enable real responses."
                    .to_string(),
            },
        })
    }
}
