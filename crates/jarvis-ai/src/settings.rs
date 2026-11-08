//! Persisted (non-secret) AI settings. Actual key *values* never appear
//! here — they live in the OS credential store (`jarvis_core::secrets`),
//! addressed by `(provider, slot.id)`. This struct only records which
//! slots exist, their labels, and whether they're enabled, plus the
//! user's model-selection preferences. It is `Serialize`/`Deserialize` so
//! the desktop app can persist it as a JSON blob via
//! `jarvis_memory::MemoryStore::{set_memory,get_memory}`.

use crate::selection::SelectionSettings;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeySlotConfig {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSettings {
    pub default_model: String,
    pub timeout_secs: u64,
    pub key_slots: Vec<KeySlotConfig>,
}

impl ProviderSettings {
    fn with_default_model(default_model: impl Into<String>) -> Self {
        Self {
            default_model: default_model.into(),
            timeout_secs: 30,
            key_slots: (1..=5)
                .map(|i| KeySlotConfig {
                    id: format!("k{i}"),
                    label: format!("Key {i}"),
                    enabled: true,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JarvisAiSettings {
    pub selection: SelectionSettings,
    pub openrouter: ProviderSettings,
    pub nvidia: ProviderSettings,
}

impl Default for JarvisAiSettings {
    fn default() -> Self {
        Self {
            selection: SelectionSettings::default(),
            openrouter: ProviderSettings::with_default_model("openrouter/auto"),
            nvidia: ProviderSettings::with_default_model("meta/llama-3.1-8b-instruct"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_have_five_key_slots_per_provider() {
        let settings = JarvisAiSettings::default();
        assert_eq!(settings.openrouter.key_slots.len(), 5);
        assert_eq!(settings.nvidia.key_slots.len(), 5);
        assert_eq!(
            settings.openrouter.key_slots.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(),
            vec!["k1", "k2", "k3", "k4", "k5"]
        );
    }

    #[test]
    fn round_trips_through_json() {
        let settings = JarvisAiSettings::default();
        let json = serde_json::to_string(&settings).expect("serialize");
        let decoded: JarvisAiSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.openrouter.key_slots, settings.openrouter.key_slots);
    }
}
