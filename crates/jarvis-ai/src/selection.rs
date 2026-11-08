//! Task-based model ranking and the user-facing selection settings that
//! constrain it. Pure logic — no network calls — so it's fully unit
//! testable without mocking a provider.

use crate::models::ModelInfo;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskCategory {
    Conversation,
    Reasoning,
    Coding,
    Image,
    ToolUse,
}

impl TaskCategory {
    fn is_compatible(self, model: &ModelInfo) -> bool {
        match self {
            TaskCategory::Conversation | TaskCategory::Reasoning | TaskCategory::Coding => {
                model.supports_text
            }
            TaskCategory::Image => model.supports_image,
            TaskCategory::ToolUse => model.supports_tools,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionSettings {
    pub auto_select: bool,
    pub preferred_provider: Option<String>,
    pub preferred_model: Option<String>,
    pub free_only: bool,
    pub max_retries: u32,
    /// Must be explicitly set to `true` before any non-free model is ever
    /// used. Defaults to `false` — paid usage is always opt-in.
    pub allow_paid: bool,
}

impl Default for SelectionSettings {
    fn default() -> Self {
        Self {
            auto_select: true,
            preferred_provider: None,
            preferred_model: None,
            free_only: true,
            max_retries: 3,
            allow_paid: false,
        }
    }
}

impl SelectionSettings {
    fn cost_eligible(&self, model: &ModelInfo) -> bool {
        if self.free_only || !self.allow_paid {
            model.is_free()
        } else {
            true
        }
    }
}

/// Returns compatible, cost-eligible, available models for `task`, most
/// preferred first: preferred model, then preferred provider, then
/// already-tested-successfully, then larger context window.
pub fn rank_candidates<'a>(
    models: &'a [ModelInfo],
    task: TaskCategory,
    settings: &SelectionSettings,
) -> Vec<&'a ModelInfo> {
    let mut candidates: Vec<&ModelInfo> = models
        .iter()
        .filter(|m| m.available)
        .filter(|m| task.is_compatible(m))
        .filter(|m| settings.cost_eligible(m))
        .collect();

    candidates.sort_by(|a, b| {
        let a_model_match = settings.preferred_model.as_deref() == Some(a.model_id.as_str());
        let b_model_match = settings.preferred_model.as_deref() == Some(b.model_id.as_str());
        let a_provider_match = settings.preferred_provider.as_deref() == Some(a.provider.as_str());
        let b_provider_match = settings.preferred_provider.as_deref() == Some(b.provider.as_str());

        b_model_match
            .cmp(&a_model_match)
            .then_with(|| b_provider_match.cmp(&a_provider_match))
            .then_with(|| b.last_tested.is_some().cmp(&a.last_tested.is_some()))
            .then_with(|| b.context_length.unwrap_or(0).cmp(&a.context_length.unwrap_or(0)))
    });

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    fn model(provider: &str, id: &str, free: bool, image: bool) -> ModelInfo {
        ModelInfo {
            provider: provider.to_string(),
            model_id: id.to_string(),
            display_name: id.to_string(),
            input_price_per_million: Some(if free { 0.0 } else { 2.0 }),
            output_price_per_million: Some(if free { 0.0 } else { 2.0 }),
            context_length: Some(8192),
            supports_text: true,
            supports_image: image,
            supports_audio: false,
            supports_tools: false,
            available: true,
            last_tested: None,
            last_refreshed: 0,
        }
    }

    #[test]
    fn excludes_paid_models_by_default() {
        let models = vec![model("openrouter", "free-a", true, false), model("openrouter", "paid-b", false, false)];
        let ranked = rank_candidates(&models, TaskCategory::Conversation, &SelectionSettings::default());
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].model_id, "free-a");
    }

    #[test]
    fn allow_paid_requires_explicit_opt_in_and_free_only_off() {
        let models = vec![model("openrouter", "paid-b", false, false)];
        let mut settings = SelectionSettings {
            allow_paid: true,
            free_only: false,
            ..SelectionSettings::default()
        };
        assert_eq!(rank_candidates(&models, TaskCategory::Conversation, &settings).len(), 1);

        settings.free_only = true; // free_only still wins even if allow_paid is true
        assert_eq!(rank_candidates(&models, TaskCategory::Conversation, &settings).len(), 0);
    }

    #[test]
    fn image_task_filters_out_text_only_models() {
        let models = vec![model("openrouter", "text-only", true, false), model("openrouter", "vision", true, true)];
        let ranked = rank_candidates(&models, TaskCategory::Image, &SelectionSettings::default());
        assert_eq!(ranked.len(), 1);
        assert_eq!(ranked[0].model_id, "vision");
    }

    #[test]
    fn preferred_model_is_ranked_first() {
        let models = vec![model("openrouter", "a", true, false), model("openrouter", "b", true, false)];
        let settings = SelectionSettings {
            preferred_model: Some("b".to_string()),
            ..SelectionSettings::default()
        };
        let ranked = rank_candidates(&models, TaskCategory::Conversation, &settings);
        assert_eq!(ranked[0].model_id, "b");
    }

    #[test]
    fn unknown_pricing_is_excluded_unless_paid_is_allowed() {
        let mut m = model("nvidia", "mystery", true, false);
        m.input_price_per_million = None;
        m.output_price_per_million = None;
        let models = vec![m];

        assert_eq!(rank_candidates(&models, TaskCategory::Conversation, &SelectionSettings::default()).len(), 0);

        let settings = SelectionSettings { allow_paid: true, free_only: false, ..SelectionSettings::default() };
        assert_eq!(rank_candidates(&models, TaskCategory::Conversation, &settings).len(), 1);
    }
}
