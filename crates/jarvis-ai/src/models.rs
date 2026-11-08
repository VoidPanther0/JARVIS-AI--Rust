//! Unified model registry and catalog discovery.
//!
//! Each provider exposes a different amount of metadata:
//! - **OpenRouter**'s `/models` endpoint is public and includes per-model
//!   pricing, context length, and input modalities, so free models can be
//!   identified with confidence.
//! - **NVIDIA**'s `/v1/models` endpoint is also public but returns only
//!   `id`/`owned_by`/`created` — no pricing or capability metadata. It is
//!   not possible to determine from this API alone whether a given NVIDIA
//!   model is free, so NVIDIA models are recorded with unknown pricing
//!   and therefore never treated as free by [`ModelInfo::is_free`].

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A single model as known to the unified registry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelInfo {
    pub provider: String,
    pub model_id: String,
    pub display_name: String,
    /// USD per 1M input tokens. `None` means pricing is unknown (not
    /// necessarily free) — see the module docs re: NVIDIA.
    pub input_price_per_million: Option<f64>,
    pub output_price_per_million: Option<f64>,
    pub context_length: Option<u32>,
    pub supports_text: bool,
    pub supports_image: bool,
    pub supports_audio: bool,
    pub supports_tools: bool,
    pub available: bool,
    /// Unix seconds of the last successful connection test, if any.
    pub last_tested: Option<u64>,
    /// Unix seconds this entry was last refreshed from the provider.
    pub last_refreshed: u64,
}

impl ModelInfo {
    /// True only when both input and output pricing are *confirmed* zero.
    /// Unknown pricing (`None`) is never treated as free.
    pub fn is_free(&self) -> bool {
        self.input_price_per_million == Some(0.0) && self.output_price_per_million == Some(0.0)
    }

    pub fn has_known_pricing(&self) -> bool {
        self.input_price_per_million.is_some() && self.output_price_per_million.is_some()
    }
}

/// In-memory + locally-cached catalog of models across providers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelRegistry {
    pub models: Vec<ModelInfo>,
}

impl ModelRegistry {
    pub fn free_models(&self) -> impl Iterator<Item = &ModelInfo> {
        self.models.iter().filter(|m| m.available && m.is_free())
    }

    pub fn by_provider<'a>(&'a self, provider: &'a str) -> impl Iterator<Item = &'a ModelInfo> {
        self.models.iter().filter(move |m| m.provider == provider)
    }

    /// Replaces every entry for `provider` with `fresh`, keeping other
    /// providers' entries untouched. Used after a per-provider refresh.
    pub fn replace_provider(&mut self, provider: &str, fresh: Vec<ModelInfo>) {
        self.models.retain(|m| m.provider != provider);
        self.models.extend(fresh);
    }

    pub fn mark_tested(&mut self, provider: &str, model_id: &str, success: bool) {
        if let Some(m) = self
            .models
            .iter_mut()
            .find(|m| m.provider == provider && m.model_id == model_id)
        {
            if success {
                m.last_tested = Some(now_unix());
            }
            m.available = success || m.available;
        }
    }
}

// ---- OpenRouter catalog ----

#[derive(Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Deserialize)]
struct OpenRouterModel {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    context_length: Option<u32>,
    pricing: OpenRouterPricing,
    #[serde(default)]
    architecture: OpenRouterArchitecture,
    #[serde(default)]
    supported_parameters: Vec<String>,
}

#[derive(Deserialize)]
struct OpenRouterPricing {
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    completion: Option<String>,
}

#[derive(Deserialize, Default)]
struct OpenRouterArchitecture {
    #[serde(default)]
    input_modalities: Vec<String>,
    #[serde(default)]
    output_modalities: Vec<String>,
}

/// USD-per-token string (e.g. `"0.0000002"` or `"0"`) -> USD per 1M
/// tokens. Unparseable or missing values become `None` (unknown), never
/// `Some(0.0)`, so they are never mistaken for a confirmed-free price.
fn parse_per_million(raw: Option<&str>) -> Option<f64> {
    raw.and_then(|s| s.parse::<f64>().ok()).map(|v| v * 1_000_000.0)
}

impl From<OpenRouterModel> for ModelInfo {
    fn from(m: OpenRouterModel) -> Self {
        let supports_tools = m
            .supported_parameters
            .iter()
            .any(|p| p == "tools" || p == "tool_choice");
        let supports_image = m
            .architecture
            .input_modalities
            .iter()
            .any(|modality| modality == "image");
        let supports_audio = m
            .architecture
            .input_modalities
            .iter()
            .any(|modality| modality == "audio")
            || m
                .architecture
                .output_modalities
                .iter()
                .any(|modality| modality == "audio");

        ModelInfo {
            provider: "openrouter".to_string(),
            display_name: m.name.unwrap_or_else(|| m.id.clone()),
            model_id: m.id,
            input_price_per_million: parse_per_million(m.pricing.prompt.as_deref()),
            output_price_per_million: parse_per_million(m.pricing.completion.as_deref()),
            context_length: m.context_length,
            supports_text: true,
            supports_image,
            supports_audio,
            supports_tools,
            available: true,
            last_tested: None,
            last_refreshed: now_unix(),
        }
    }
}

/// Fetches the full OpenRouter model catalog (public endpoint, no API key
/// required) and converts it into [`ModelInfo`] entries.
pub async fn fetch_openrouter_catalog(
    client: &reqwest::Client,
) -> Result<Vec<ModelInfo>, crate::error::AiError> {
    let resp = client
        .get("https://openrouter.ai/api/v1/models")
        .send()
        .await
        .map_err(|e| crate::error::AiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(crate::error::AiError::Network(format!(
            "OpenRouter models request failed: HTTP {}",
            resp.status()
        )));
    }

    let body: OpenRouterModelsResponse = resp
        .json()
        .await
        .map_err(|e| crate::error::AiError::Network(format!("invalid OpenRouter response: {e}")))?;

    Ok(body.data.into_iter().map(ModelInfo::from).collect())
}

// ---- NVIDIA catalog ----

#[derive(Deserialize)]
struct NvidiaModelsResponse {
    data: Vec<NvidiaModel>,
}

#[derive(Deserialize)]
struct NvidiaModel {
    id: String,
}

/// Fetches the NVIDIA model ID list. The public `/v1/models` endpoint
/// returns no pricing or capability metadata, so every entry is recorded
/// with unknown pricing/capabilities rather than guessed values — see the
/// module docs.
pub async fn fetch_nvidia_catalog(
    client: &reqwest::Client,
) -> Result<Vec<ModelInfo>, crate::error::AiError> {
    let resp = client
        .get("https://integrate.api.nvidia.com/v1/models")
        .send()
        .await
        .map_err(|e| crate::error::AiError::Network(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(crate::error::AiError::Network(format!(
            "NVIDIA models request failed: HTTP {}",
            resp.status()
        )));
    }

    let body: NvidiaModelsResponse = resp
        .json()
        .await
        .map_err(|e| crate::error::AiError::Network(format!("invalid NVIDIA response: {e}")))?;

    let now = now_unix();
    Ok(body
        .data
        .into_iter()
        .map(|m| ModelInfo {
            provider: "nvidia".to_string(),
            display_name: m.id.clone(),
            model_id: m.id,
            input_price_per_million: None,
            output_price_per_million: None,
            context_length: None,
            supports_text: true,
            supports_image: false,
            supports_audio: false,
            supports_tools: false,
            available: true,
            last_tested: None,
            last_refreshed: now,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn free_model(provider: &str, id: &str) -> ModelInfo {
        ModelInfo {
            provider: provider.to_string(),
            model_id: id.to_string(),
            display_name: id.to_string(),
            input_price_per_million: Some(0.0),
            output_price_per_million: Some(0.0),
            context_length: Some(8192),
            supports_text: true,
            supports_image: false,
            supports_audio: false,
            supports_tools: false,
            available: true,
            last_tested: None,
            last_refreshed: now_unix(),
        }
    }

    #[test]
    fn unknown_pricing_is_never_free() {
        let mut m = free_model("nvidia", "some/model");
        m.input_price_per_million = None;
        m.output_price_per_million = None;
        assert!(!m.is_free());
        assert!(!m.has_known_pricing());
    }

    #[test]
    fn zero_priced_model_is_free() {
        assert!(free_model("openrouter", "x/y:free").is_free());
    }

    #[test]
    fn paid_model_is_not_free() {
        let mut m = free_model("openrouter", "x/y");
        m.input_price_per_million = Some(1.5);
        assert!(!m.is_free());
    }

    #[test]
    fn replace_provider_only_touches_that_providers_entries() {
        let mut registry = ModelRegistry {
            models: vec![free_model("openrouter", "a"), free_model("nvidia", "b")],
        };
        registry.replace_provider("openrouter", vec![free_model("openrouter", "c")]);
        let ids: Vec<&str> = registry.models.iter().map(|m| m.model_id.as_str()).collect();
        assert_eq!(ids, vec!["b", "c"]);
    }

    #[test]
    fn parse_per_million_handles_zero_and_fractional_and_missing() {
        assert_eq!(parse_per_million(Some("0")), Some(0.0));
        assert_eq!(parse_per_million(Some("0.000001")), Some(1.0));
        assert_eq!(parse_per_million(None), None);
        assert_eq!(parse_per_million(Some("not-a-number")), None);
    }
}
