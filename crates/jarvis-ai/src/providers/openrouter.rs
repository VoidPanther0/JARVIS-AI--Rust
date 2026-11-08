use crate::error::AiResult;
use crate::provider::{AiProvider, ChatRequest, ChatResponse};
use crate::providers::OpenAiCompatibleClient;
use async_trait::async_trait;
use std::time::Duration;

pub const BASE_URL: &str = "https://openrouter.ai/api/v1";

/// OpenRouter chat completions, OpenAI-compatible. Model discovery lives
/// in `crate::models::fetch_openrouter_catalog` (public endpoint, no key
/// needed); this type only sends chat requests, which do require a key.
pub struct OpenRouterProvider {
    inner: OpenAiCompatibleClient,
}

impl OpenRouterProvider {
    pub fn new(default_model: impl Into<String>, timeout: Duration) -> AiResult<Self> {
        Ok(Self {
            inner: OpenAiCompatibleClient::new("openrouter", BASE_URL, default_model, timeout)?,
        })
    }

    pub async fn test_connection(&self, api_key: &str) -> AiResult<()> {
        self.inner.test_connection(api_key).await
    }
}

#[async_trait]
impl AiProvider for OpenRouterProvider {
    fn name(&self) -> &str {
        "openrouter"
    }

    async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse> {
        self.inner.chat(request).await
    }
}
