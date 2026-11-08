//! Ties together key rotation, the model registry, and task-based
//! selection into one entry point the desktop app calls: [`Orchestrator::send_chat`].
//!
//! Failure handling policy (see `AiError` for the variants involved):
//! on a rate limit the offending key is put on cooldown and the next
//! ranked candidate model is tried; on any other provider error the
//! candidate is recorded as a failed test and the next one is tried; a
//! cancellation stops immediately rather than continuing to retry; and
//! once `max_retries` candidates have been tried without success, the
//! orchestrator reports [`AiError::NoModelAvailable`] rather than ever
//! silently falling back to a paid model.
//!
//! There are exactly two real providers today, so each gets its own
//! field rather than going through a `dyn AiProvider` + downcast — that
//! would add a layer of indirection for no real benefit at this size.

use crate::error::{AiError, AiResult};
use crate::keys::{ApiKeySlot, KeyPool};
use crate::models::{fetch_nvidia_catalog, fetch_openrouter_catalog, ModelInfo, ModelRegistry};
use crate::provider::{AiProvider, ChatRequest, ChatResponse};
use crate::providers::nvidia::NvidiaProvider;
use crate::providers::openrouter::OpenRouterProvider;
use crate::selection::{rank_candidates, SelectionSettings, TaskCategory};
use crate::settings::{JarvisAiSettings, ProviderSettings};
use std::time::Duration;
use tokio::sync::{oneshot, Mutex, RwLock};

struct ProviderHandle<P> {
    provider: P,
    keys: Mutex<KeyPool>,
}

pub struct Orchestrator {
    http: reqwest::Client,
    openrouter: RwLock<Option<ProviderHandle<OpenRouterProvider>>>,
    nvidia: RwLock<Option<ProviderHandle<NvidiaProvider>>>,
    registry: RwLock<ModelRegistry>,
    settings: RwLock<SelectionSettings>,
}

/// Loads the enabled key slots for `provider` from the OS credential
/// store into a fresh [`KeyPool`]. Slots with no stored secret yet (the
/// user hasn't filled that one in) are skipped, not treated as an error.
pub fn load_key_pool(provider: &str, settings: &ProviderSettings) -> AiResult<KeyPool> {
    let mut slots = Vec::new();
    for slot in &settings.key_slots {
        if !slot.enabled {
            continue;
        }
        match jarvis_core::secrets::get(provider, &slot.id) {
            Ok(Some(secret)) => slots.push(ApiKeySlot::new(slot.id.clone(), secret)),
            Ok(None) => {}
            Err(e) => {
                return Err(AiError::ProviderFailure {
                    provider: provider.to_string(),
                    message: format!("could not read stored key '{}': {e}", slot.id),
                })
            }
        }
    }
    Ok(KeyPool::new(slots))
}

impl Orchestrator {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            openrouter: RwLock::new(None),
            nvidia: RwLock::new(None),
            registry: RwLock::new(ModelRegistry::default()),
            settings: RwLock::new(SelectionSettings::default()),
        }
    }

    /// (Re)builds both cloud providers and their key pools from
    /// `settings`. Call this at startup and whenever the user changes
    /// provider configuration or keys.
    pub async fn configure(&self, settings: &JarvisAiSettings) -> AiResult<()> {
        *self.settings.write().await = settings.selection.clone();

        let openrouter_keys = load_key_pool("openrouter", &settings.openrouter)?;
        let nvidia_keys = load_key_pool("nvidia", &settings.nvidia)?;

        let openrouter = OpenRouterProvider::new(
            settings.openrouter.default_model.clone(),
            Duration::from_secs(settings.openrouter.timeout_secs),
        )?;
        let nvidia = NvidiaProvider::new(
            settings.nvidia.default_model.clone(),
            Duration::from_secs(settings.nvidia.timeout_secs),
        )?;

        *self.openrouter.write().await = Some(ProviderHandle {
            provider: openrouter,
            keys: Mutex::new(openrouter_keys),
        });
        *self.nvidia.write().await = Some(ProviderHandle {
            provider: nvidia,
            keys: Mutex::new(nvidia_keys),
        });
        Ok(())
    }

    pub async fn selection_settings(&self) -> SelectionSettings {
        self.settings.read().await.clone()
    }

    pub async fn models_snapshot(&self) -> ModelRegistry {
        self.registry.read().await.clone()
    }

    /// Refreshes the model catalog for one provider from its real API and
    /// returns how many models were discovered for it.
    pub async fn refresh_models(&self, provider: &str) -> AiResult<usize> {
        let fresh: Vec<ModelInfo> = match provider {
            "openrouter" => fetch_openrouter_catalog(&self.http).await?,
            "nvidia" => fetch_nvidia_catalog(&self.http).await?,
            other => return Err(AiError::ProviderNotFound(other.to_string())),
        };
        let count = fresh.len();
        self.registry.write().await.replace_provider(provider, fresh);
        Ok(count)
    }

    /// Sends one real request through that provider's current key pool to
    /// confirm the key and default model actually work.
    pub async fn test_connection(&self, provider: &str) -> AiResult<()> {
        match provider {
            "openrouter" => {
                let guard = self.openrouter.read().await;
                let handle = guard
                    .as_ref()
                    .ok_or_else(|| AiError::NoKeyConfigured(provider.to_string()))?;
                let (key_id, secret) = handle
                    .keys
                    .lock()
                    .await
                    .next_key()
                    .ok_or_else(|| AiError::NoKeyConfigured(provider.to_string()))?;
                let result = handle.provider.test_connection(&secret).await;
                if let Err(AiError::RateLimited { .. }) = &result {
                    handle
                        .keys
                        .lock()
                        .await
                        .mark_rate_limited(&key_id, Duration::from_secs(30));
                }
                result
            }
            "nvidia" => {
                let guard = self.nvidia.read().await;
                let handle = guard
                    .as_ref()
                    .ok_or_else(|| AiError::NoKeyConfigured(provider.to_string()))?;
                let (key_id, secret) = handle
                    .keys
                    .lock()
                    .await
                    .next_key()
                    .ok_or_else(|| AiError::NoKeyConfigured(provider.to_string()))?;
                let result = handle.provider.test_connection(&secret).await;
                if let Err(AiError::RateLimited { .. }) = &result {
                    handle
                        .keys
                        .lock()
                        .await
                        .mark_rate_limited(&key_id, Duration::from_secs(30));
                }
                result
            }
            other => Err(AiError::ProviderNotFound(other.to_string())),
        }
    }

    /// Core entry point: ranks compatible, cost-eligible models for
    /// `task`, then tries up to `max_retries` of them in order, rotating
    /// to a fresh key for each attempt. Stops immediately if `cancel`
    /// fires. Never uses a paid model unless the user's settings already
    /// allow it (`rank_candidates` enforces that, not this loop).
    pub async fn send_chat(
        &self,
        task: TaskCategory,
        mut request: ChatRequest,
        mut cancel: oneshot::Receiver<()>,
    ) -> AiResult<(ChatResponse, String, String)> {
        let settings = self.settings.read().await.clone();
        let ranked: Vec<ModelInfo> = {
            let registry = self.registry.read().await;
            rank_candidates(&registry.models, task, &settings)
                .into_iter()
                .cloned()
                .collect()
        };

        if ranked.is_empty() {
            return Err(AiError::NoModelAvailable {
                provider: settings
                    .preferred_provider
                    .unwrap_or_else(|| "any".to_string()),
                task: format!("{task:?}"),
                attempted: 0,
            });
        }

        let max_attempts = (settings.max_retries.max(1) as usize).min(ranked.len());
        let mut attempted = 0usize;
        let mut last_provider = String::new();

        for candidate in ranked.into_iter().take(max_attempts) {
            attempted += 1;
            last_provider = candidate.provider.clone();
            request.model = Some(candidate.model_id.clone());

            let outcome: AiResult<ChatResponse> = match candidate.provider.as_str() {
                "openrouter" => {
                    let guard = self.openrouter.read().await;
                    let Some(handle) = guard.as_ref() else {
                        continue;
                    };
                    let Some((key_id, secret)) = handle.keys.lock().await.next_key() else {
                        continue;
                    };
                    request.api_key = Some(secret);
                    let attempt = handle.provider.chat(request.clone());
                    let result = tokio::select! {
                        r = attempt => r,
                        _ = &mut cancel => Err(AiError::Cancelled { provider: candidate.provider.clone() }),
                    };
                    if let Err(AiError::RateLimited { .. }) = &result {
                        handle
                            .keys
                            .lock()
                            .await
                            .mark_rate_limited(&key_id, Duration::from_secs(30));
                    }
                    result
                }
                "nvidia" => {
                    let guard = self.nvidia.read().await;
                    let Some(handle) = guard.as_ref() else {
                        continue;
                    };
                    let Some((key_id, secret)) = handle.keys.lock().await.next_key() else {
                        continue;
                    };
                    request.api_key = Some(secret);
                    let attempt = handle.provider.chat(request.clone());
                    let result = tokio::select! {
                        r = attempt => r,
                        _ = &mut cancel => Err(AiError::Cancelled { provider: candidate.provider.clone() }),
                    };
                    if let Err(AiError::RateLimited { .. }) = &result {
                        handle
                            .keys
                            .lock()
                            .await
                            .mark_rate_limited(&key_id, Duration::from_secs(30));
                    }
                    result
                }
                _ => continue,
            };

            match outcome {
                Ok(response) => {
                    self.registry
                        .write()
                        .await
                        .mark_tested(&candidate.provider, &candidate.model_id, true);
                    return Ok((response, candidate.provider, candidate.model_id));
                }
                Err(AiError::Cancelled { provider }) => {
                    return Err(AiError::Cancelled { provider });
                }
                Err(_) => {
                    self.registry
                        .write()
                        .await
                        .mark_tested(&candidate.provider, &candidate.model_id, false);
                }
            }
        }

        Err(AiError::NoModelAvailable {
            provider: last_provider,
            task: format!("{task:?}"),
            attempted,
        })
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        Self::new()
    }
}
