//! Smart caching system for LLM responses using BLAKE3 hashing.

use super::error::{LlmError, Result};
use crate::ExportedSymbol;
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Cached LLM response data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedResponse {
    /// LLM-generated explanation.
    pub explanation: String,

    /// LLM-generated code examples.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,

    /// LLM-generated best practices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub best_practices: Vec<String>,

    /// Timestamp when this was cached.
    pub timestamp: String,

    /// Model that generated this response.
    pub model: String,
}

/// LLM response cache with BLAKE3-based key generation.
///
/// The cache automatically invalidates when:
/// - Source code changes (file content hash changes)
/// - Symbol definition changes (name, kind, location)
/// - JSDoc comments change
/// - Model name changes
pub struct LlmCache {
    /// Cache directory path (typically `.fob-cache/docs-llm/`).
    cache_dir: PathBuf,

    /// Whether caching is enabled.
    enabled: bool,
}

impl LlmCache {
    /// Creates a new cache instance.
    ///
    /// # Arguments
    ///
    /// * `cache_dir` - Directory to store cache files
    /// * `enabled` - Whether caching is enabled
    pub fn new(cache_dir: impl AsRef<Path>, enabled: bool) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            enabled,
        }
    }

    /// Generates a cache key for a symbol.
    ///
    /// The key is a BLAKE3 hash of:
    /// - Symbol name
    /// - Symbol kind
    /// - Symbol location (line, column)
    /// - File content (detects code changes)
    /// - Existing JSDoc summary (detects doc changes)
    /// - Model name (invalidates on model change)
    ///
    /// This ensures the cache automatically invalidates when any relevant input changes.
    pub fn cache_key(
        &self,
        symbol: &ExportedSymbol,
        file_content: &str,
        model: &str,
    ) -> String {
        let mut hasher = Hasher::new();

        // Hash symbol metadata
        hasher.update(symbol.name.as_bytes());
        hasher.update(format!("{:?}", symbol.kind).as_bytes());
        hasher.update(&symbol.location.line.to_le_bytes());
        hasher.update(&symbol.location.column.to_le_bytes());

        // Hash file content (detects code changes)
        hasher.update(file_content.as_bytes());

        // Hash existing JSDoc (detects doc changes)
        if let Some(summary) = &symbol.summary {
            hasher.update(summary.as_bytes());
        }
        for param in &symbol.parameters {
            hasher.update(param.name.as_bytes());
            if let Some(desc) = &param.description {
                hasher.update(desc.as_bytes());
            }
        }
        if let Some(returns) = &symbol.returns {
            hasher.update(returns.as_bytes());
        }

        // Hash model name (invalidates on model change)
        hasher.update(model.as_bytes());

        format!("{}.json", hasher.finalize().to_hex())
    }

    /// Retrieves a cached response if available.
    ///
    /// Returns `Ok(None)` if cache is disabled or no cached entry exists.
    pub fn get(&self, key: &str) -> Result<Option<CachedResponse>> {
        if !self.enabled {
            return Ok(None);
        }

        let path = self.cache_dir.join(key);
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&path).map_err(|e| LlmError::CacheError {
            operation: format!("read cache file: {}", path.display()),
            source: Box::new(e),
        })?;

        let cached: CachedResponse = serde_json::from_str(&content)?;

        Ok(Some(cached))
    }

    /// Stores a response in the cache.
    ///
    /// No-op if caching is disabled.
    pub fn set(&self, key: &str, response: &CachedResponse) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        std::fs::create_dir_all(&self.cache_dir).map_err(|e| LlmError::CacheError {
            operation: format!("create cache directory: {}", self.cache_dir.display()),
            source: Box::new(e),
        })?;

        let path = self.cache_dir.join(key);
        let json = serde_json::to_string_pretty(response)?;

        std::fs::write(&path, json).map_err(|e| LlmError::CacheError {
            operation: format!("write cache file: {}", path.display()),
            source: Box::new(e),
        })?;

        Ok(())
    }

    /// Clears all cached entries.
    ///
    /// No-op if caching is disabled.
    pub fn clear(&self) -> Result<()> {
        if !self.enabled || !self.cache_dir.exists() {
            return Ok(());
        }

        std::fs::remove_dir_all(&self.cache_dir).map_err(|e| LlmError::CacheError {
            operation: format!("clear cache directory: {}", self.cache_dir.display()),
            source: Box::new(e),
        })?;

        Ok(())
    }

    /// Returns cache statistics.
    pub fn stats(&self) -> Result<CacheStats> {
        if !self.enabled || !self.cache_dir.exists() {
            return Ok(CacheStats {
                enabled: self.enabled,
                total_entries: 0,
                total_size_bytes: 0,
            });
        }

        let mut total_entries = 0;
        let mut total_size_bytes = 0;

        for entry in std::fs::read_dir(&self.cache_dir).map_err(|e| LlmError::CacheError {
            operation: format!("read cache directory: {}", self.cache_dir.display()),
            source: Box::new(e),
        })? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                total_entries += 1;
                total_size_bytes += entry.metadata()?.len();
            }
        }

        Ok(CacheStats {
            enabled: self.enabled,
            total_entries,
            total_size_bytes,
        })
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Whether caching is enabled.
    pub enabled: bool,

    /// Total number of cached entries.
    pub total_entries: usize,

    /// Total size of cached data in bytes.
    pub total_size_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceLocation, SymbolKind};

    #[test]
    fn test_cache_key_deterministic() {
        let cache = LlmCache::new("/tmp/test-cache", true);
        let symbol = ExportedSymbol::new(
            "testFunc",
            SymbolKind::Function,
            SourceLocation::new(10, 5),
        );

        let key1 = cache.cache_key(&symbol, "file content", "llama3.2");
        let key2 = cache.cache_key(&symbol, "file content", "llama3.2");

        assert_eq!(key1, key2, "Cache keys should be deterministic");
    }

    #[test]
    fn test_cache_key_invalidates_on_changes() {
        let cache = LlmCache::new("/tmp/test-cache", true);
        let symbol = ExportedSymbol::new(
            "testFunc",
            SymbolKind::Function,
            SourceLocation::new(10, 5),
        );

        let key_original = cache.cache_key(&symbol, "file content", "llama3.2");

        // Change file content
        let key_changed_file = cache.cache_key(&symbol, "different content", "llama3.2");
        assert_ne!(key_original, key_changed_file);

        // Change model
        let key_changed_model = cache.cache_key(&symbol, "file content", "codellama");
        assert_ne!(key_original, key_changed_model);

        // Change symbol
        let mut symbol2 = symbol.clone();
        symbol2.name = "differentFunc".to_string();
        let key_changed_symbol = cache.cache_key(&symbol2, "file content", "llama3.2");
        assert_ne!(key_original, key_changed_symbol);
    }

    #[test]
    fn test_disabled_cache() {
        let cache = LlmCache::new("/tmp/test-cache-disabled", false);
        let response = CachedResponse {
            explanation: "test".to_string(),
            examples: vec![],
            best_practices: vec![],
            timestamp: "2025-01-15T00:00:00Z".to_string(),
            model: "llama3.2".to_string(),
        };

        // Should not error, just no-op
        assert!(cache.set("test-key", &response).is_ok());
        assert!(cache.get("test-key").unwrap().is_none());
    }
}
