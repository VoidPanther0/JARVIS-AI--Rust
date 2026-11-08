use crate::error::{AiError, AiResult};
use crate::null_provider::NullProvider;
use crate::provider::AiProvider;
use std::collections::HashMap;
use std::sync::Arc;

/// Looks up a configured `AiProvider` by name. Always has `"null"`
/// registered so the application works before any real provider exists.
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn AiProvider>>,
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        let mut registry = Self {
            providers: HashMap::new(),
        };
        registry.register(Arc::new(NullProvider));
        registry
    }
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: Arc<dyn AiProvider>) {
        self.providers.insert(provider.name().to_string(), provider);
    }

    pub fn get(&self, name: &str) -> AiResult<Arc<dyn AiProvider>> {
        self.providers
            .get(name)
            .cloned()
            .ok_or_else(|| AiError::ProviderNotFound(name.to_string()))
    }
}
