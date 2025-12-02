//! Provider registry for managing data providers

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;

use crate::frontmatter::PropDefinition;
use super::{Provider, ProviderError};

/// Registry of data providers
///
/// Manages provider registration and resolution of prop definitions.
/// Providers are stored by their ID for O(1) lookup.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use fob_mdx::{ProviderRegistry, Provider};
///
/// let mut registry = ProviderRegistry::new();
/// registry.register(Arc::new(GitHubProvider::new(None)));
/// registry.register(Arc::new(NotionProvider::new(api_key)));
///
/// // Resolve a prop
/// let value = registry.resolve(&prop_definition).await?;
/// ```
pub struct ProviderRegistry {
    providers: HashMap<String, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider
    ///
    /// Providers are keyed by their `id()` return value.
    /// Registering a provider with the same ID will replace the existing one.
    pub fn register(&mut self, provider: Arc<dyn Provider>) {
        self.providers.insert(provider.id().to_string(), provider);
    }

    /// Get a provider by ID
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Provider>> {
        self.providers.get(id)
    }

    /// Check if a provider is registered
    pub fn has(&self, id: &str) -> bool {
        self.providers.contains_key(id)
    }

    /// Get list of registered provider IDs
    pub fn ids(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }

    /// Resolve a single prop definition
    ///
    /// Looks up the provider by `prop.provider` and calls `resolve`.
    pub async fn resolve(&self, prop: &PropDefinition) -> Result<Value, ProviderError> {
        let provider = self
            .providers
            .get(&prop.provider)
            .ok_or_else(|| ProviderError::UnknownProvider(prop.provider.clone()))?;

        provider.resolve(prop).await
    }

    /// Resolve multiple prop definitions
    ///
    /// Returns a map of prop name -> resolution result.
    /// Does not short-circuit on errors; all props are attempted.
    pub async fn resolve_all(
        &self,
        props: &[PropDefinition],
    ) -> HashMap<String, Result<Value, ProviderError>> {
        let mut results = HashMap::new();

        for prop in props {
            let result = self.resolve(prop).await;
            results.insert(prop.name.clone(), result);
        }

        results
    }

    /// Resolve multiple prop definitions, returning only successes
    ///
    /// Failed resolutions are logged but not returned.
    /// Returns a map of prop name -> resolved value.
    pub async fn resolve_all_ok(&self, props: &[PropDefinition]) -> HashMap<String, Value> {
        let mut results = HashMap::new();

        for prop in props {
            match self.resolve(prop).await {
                Ok(value) => {
                    results.insert(prop.name.clone(), value);
                }
                Err(e) => {
                    eprintln!("Warning: Failed to resolve prop '{}': {}", prop.name, e);
                }
            }
        }

        results
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::PropOptions;

    // Mock provider for testing
    struct MockProvider {
        id: String,
        value: Value,
    }

    #[async_trait::async_trait]
    impl Provider for MockProvider {
        fn id(&self) -> &str {
            &self.id
        }

        async fn resolve(&self, _prop: &PropDefinition) -> Result<Value, ProviderError> {
            Ok(self.value.clone())
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry = ProviderRegistry::new();

        let provider = Arc::new(MockProvider {
            id: "test".to_string(),
            value: Value::String("hello".to_string()),
        });

        registry.register(provider);

        assert!(registry.has("test"));
        assert!(!registry.has("unknown"));
        assert!(registry.get("test").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_ids() {
        let mut registry = ProviderRegistry::new();

        registry.register(Arc::new(MockProvider {
            id: "github".to_string(),
            value: Value::Null,
        }));
        registry.register(Arc::new(MockProvider {
            id: "notion".to_string(),
            value: Value::Null,
        }));

        let ids = registry.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"github"));
        assert!(ids.contains(&"notion"));
    }

    #[tokio::test]
    async fn test_registry_resolve() {
        let mut registry = ProviderRegistry::new();

        registry.register(Arc::new(MockProvider {
            id: "test".to_string(),
            value: Value::Number(42.into()),
        }));

        let prop = PropDefinition {
            name: "data".to_string(),
            provider: "test".to_string(),
            method: "get".to_string(),
            args: vec![],
            fields: vec![],
            options: PropOptions::default(),
            raw: "test.get()".to_string(),
        };

        let result = registry.resolve(&prop).await.unwrap();
        assert_eq!(result, Value::Number(42.into()));
    }

    #[tokio::test]
    async fn test_registry_resolve_unknown_provider() {
        let registry = ProviderRegistry::new();

        let prop = PropDefinition {
            name: "data".to_string(),
            provider: "unknown".to_string(),
            method: "get".to_string(),
            args: vec![],
            fields: vec![],
            options: PropOptions::default(),
            raw: "unknown.get()".to_string(),
        };

        let result = registry.resolve(&prop).await;
        assert!(matches!(result, Err(ProviderError::UnknownProvider(_))));
    }
}
