//! HTTP provider implementations. Both OpenRouter and NVIDIA expose an
//! OpenAI-compatible `/chat/completions` endpoint, so the request/response
//! handling is shared here; `openrouter.rs` and `nvidia.rs` only supply the
//! base URL and provider name.

pub mod nvidia;
pub mod openrouter;

use crate::error::AiError;
use crate::provider::{ChatMessage, ChatRequest, ChatResponse, Role};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize)]
struct OpenAiChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessage>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
    }
}

/// Shared OpenAI-compatible chat client used by both providers.
pub(crate) struct OpenAiCompatibleClient {
    client: reqwest::Client,
    base_url: String,
    provider_name: String,
    default_model: String,
    timeout: Duration,
}

impl OpenAiCompatibleClient {
    pub(crate) fn new(
        provider_name: impl Into<String>,
        base_url: impl Into<String>,
        default_model: impl Into<String>,
        timeout: Duration,
    ) -> Result<Self, AiError> {
        let provider_name = provider_name.into();
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| AiError::ProviderFailure {
                provider: provider_name.clone(),
                message: format!("could not build HTTP client: {e}"),
            })?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            provider_name,
            default_model: default_model.into(),
            timeout,
        })
    }

    pub(crate) async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError> {
        let api_key = request
            .api_key
            .as_deref()
            .ok_or_else(|| AiError::NoKeyConfigured(self.provider_name.clone()))?;
        let model = request.model.as_deref().unwrap_or(&self.default_model);

        let messages: Vec<OpenAiMessage> = request
            .messages
            .iter()
            .map(|m| OpenAiMessage {
                role: role_str(m.role),
                content: m.content.clone(),
            })
            .collect();

        let body = OpenAiChatRequest { model, messages };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| self.map_reqwest_err(e))?;

        self.handle_response(resp).await
    }

    /// Sends a minimal real request to confirm the key and default model
    /// actually work end to end — not just that the host is reachable.
    pub(crate) async fn test_connection(&self, api_key: &str) -> Result<(), AiError> {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: Role::User,
                content: "ping".to_string(),
            }],
            model: None,
            api_key: Some(api_key.to_string()),
        };
        self.chat(request).await.map(|_| ())
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<ChatResponse, AiError> {
        let status = resp.status();

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AiError::RateLimited {
                provider: self.provider_name.clone(),
            });
        }
        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return Err(AiError::Unauthorized {
                provider: self.provider_name.clone(),
                status: status.as_u16(),
            });
        }
        if status.is_server_error() {
            let message = resp.text().await.unwrap_or_default();
            return Err(AiError::ServerError {
                provider: self.provider_name.clone(),
                status: status.as_u16(),
                message,
            });
        }
        if !status.is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(AiError::ProviderFailure {
                provider: self.provider_name.clone(),
                message: format!("HTTP {status}: {message}"),
            });
        }

        let parsed: OpenAiChatResponse = resp.json().await.map_err(|e| AiError::ProviderFailure {
            provider: self.provider_name.clone(),
            message: format!("invalid response body: {e}"),
        })?;

        let choice = parsed
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| AiError::ProviderFailure {
                provider: self.provider_name.clone(),
                message: "response contained no choices".to_string(),
            })?;

        Ok(ChatResponse {
            message: ChatMessage {
                role: Role::Assistant,
                content: choice.message.content,
            },
        })
    }

    fn map_reqwest_err(&self, e: reqwest::Error) -> AiError {
        if e.is_timeout() {
            AiError::Timeout {
                provider: self.provider_name.clone(),
                timeout_secs: self.timeout.as_secs(),
            }
        } else {
            AiError::Network(e.to_string())
        }
    }
}
