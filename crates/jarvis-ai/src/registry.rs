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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_has_null_provider() {
        let registry = ProviderRegistry::default();
        let provider = registry
            .get("null")
            .expect("null provider is always registered");
        assert_eq!(provider.name(), "null");
    }

    #[test]
    fn unknown_provider_name_is_an_error() {
        let registry = ProviderRegistry::new();
        match registry.get("does-not-exist") {
            Err(AiError::ProviderNotFound(name)) => assert_eq!(name, "does-not-exist"),
            Ok(_) => panic!("expected ProviderNotFound, got Ok"),
            Err(_) => panic!("expected ProviderNotFound, got a different error"),
        }
    }
}
