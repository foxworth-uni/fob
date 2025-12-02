//! Provider trait definition

use async_trait::async_trait;
use serde_json::Value;

use crate::frontmatter::PropDefinition;
use super::ProviderError;

/// A data provider that can resolve prop definitions
///
/// Providers fetch external data (from APIs, databases, etc.) and
/// return JSON values that are injected into MDX pages.
///
/// # Implementation Notes
///
/// - Providers must be `Send + Sync` for use in async contexts
/// - The `resolve` method receives a full `PropDefinition` including method and args
/// - Field extraction (via `prop.fields`) should be handled by the provider
///
/// # Example
///
/// ```ignore
/// use async_trait::async_trait;
/// use fob_mdx::{Provider, ProviderError, PropDefinition};
/// use serde_json::Value;
///
/// struct GitHubProvider {
///     client: reqwest::Client,
///     token: Option<String>,
/// }
///
/// #[async_trait]
/// impl Provider for GitHubProvider {
///     fn id(&self) -> &str {
///         "github"
///     }
///
///     async fn resolve(&self, prop: &PropDefinition) -> Result<Value, ProviderError> {
///         match prop.method.as_str() {
///             "repo" => self.fetch_repo(prop).await,
///             "user" => self.fetch_user(prop).await,
///             _ => Err(ProviderError::resolution(
///                 "github",
///                 &prop.method,
///                 format!("Unknown method: {}", prop.method),
///             )),
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait Provider: Send + Sync {
    /// Provider identifier (e.g., "github", "notion", "posthog")
    ///
    /// This must match the provider name used in MDX frontmatter expressions.
    fn id(&self) -> &str;

    /// Resolve a prop definition to a JSON value
    ///
    /// # Arguments
    ///
    /// * `prop` - The prop definition to resolve, containing:
    ///   - `method`: The method to call (e.g., "repo", "database")
    ///   - `args`: Arguments passed to the method
    ///   - `fields`: Field chain to extract from the response
    ///   - `options.strategy`: When this data should be fetched
    ///
    /// # Returns
    ///
    /// The resolved JSON value, or an error if resolution fails.
    /// If `prop.fields` is non-empty, the provider should extract
    /// the nested field from the response.
    async fn resolve(&self, prop: &PropDefinition) -> Result<Value, ProviderError>;
}
