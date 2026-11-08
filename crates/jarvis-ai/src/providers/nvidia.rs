use crate::error::AiResult;
use crate::provider::{AiProvider, ChatRequest, ChatResponse};
use crate::providers::OpenAiCompatibleClient;
use async_trait::async_trait;
use std::time::Duration;

pub const BASE_URL: &str = "https://integrate.api.nvidia.com/v1";

/// NVIDIA NIM chat completions, OpenAI-compatible. Unlike OpenRouter,
/// NVIDIA's public model list has no pricing metadata (see
/// `crate::models`), so free/paid status for a given model is unknown
/// until confirmed another way (e.g. NVIDIA's own documentation).
pub struct NvidiaProvider {
    inner: OpenAiCompatibleClient,
}

impl NvidiaProvider {
    pub fn new(default_model: impl Into<String>, timeout: Duration) -> AiResult<Self> {
        Ok(Self {
            inner: OpenAiCompatibleClient::new("nvidia", BASE_URL, default_model, timeout)?,
        })
    }

    pub async fn test_connection(&self, api_key: &str) -> AiResult<()> {
        self.inner.test_connection(api_key).await
    }
}

#[async_trait]
impl AiProvider for NvidiaProvider {
    fn name(&self) -> &str {
        "nvidia"
    }

    async fn chat(&self, request: ChatRequest) -> AiResult<ChatResponse> {
        self.inner.chat(request).await
    }
}
