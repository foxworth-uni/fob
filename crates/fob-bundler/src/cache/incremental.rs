//! Incremental module graph caching.
//!
//! Caches the module graph between builds to avoid re-analysis when files haven't changed.
//! This is complementary to the build cache - build cache skips Rolldown entirely on cache
//! hit, while incremental cache only skips graph re-analysis (Rolldown still runs).

use fob_graph::ModuleGraph;
use rustc_hash::FxHashMap as HashMap;
use std::fs;
use std::path::Path;

use super::CacheError;

/// Incremental cache for module graph analysis.
///
/// Stores:
/// - Serialized module graph (post-Rolldown analysis)
/// - BLAKE3 hashes of all module file contents
/// - Rolldown version string (for invalidation)
/// - Format version (for forward compatibility)
///
/// # Cache Invalidation
///
/// The cache is invalidated if:
/// - Any file content hash changes
/// - Rolldown version changes
/// - Format version is incompatible
/// - Entry points change
#[derive(Debug, Clone)]
pub struct IncrementalCache {
    /// Cached module graph (None if cache miss or invalid).
    pub graph: Option<ModuleGraph>,

    /// BLAKE3 hashes of module file contents.
    ///
    /// Maps ModuleId to content hash. Used for change detection.
    pub module_hashes: HashMap<fob_graph::ModuleId, [u8; 32]>,

    /// Rolldown version used to create this cache.
    ///
    /// If the current Rolldown version differs, the cache is invalidated
    /// since Rolldown's analysis may have changed.
    pub rolldown_version: String,

    /// Cache format version.
    ///
    /// Incremented when the cache format changes. Incompatible versions
    /// are rejected to prevent deserialization errors.
    pub format_version: u32,
}

impl IncrementalCache {
    /// Current cache format version.
    pub const FORMAT_VERSION: u32 = 1;

    /// Create a new empty incremental cache.
    pub fn new() -> Self {
        Self {
            graph: None,
            module_hashes: HashMap::default(),
            rolldown_version: get_rolldown_version(),
            format_version: Self::FORMAT_VERSION,
        }
    }

    /// Load incremental cache from disk.
    ///
    /// Returns `Ok(None)` if the cache file doesn't exist (cold start).
    /// Returns `Ok(Some(cache))` if loaded successfully.
    /// Returns `Err` on I/O errors or format incompatibilities.
    ///
    /// # Cache File Location
    ///
    /// The cache is stored at `<dir>/incremental.bin`.
    ///
    /// # Errors
    ///
    /// - I/O errors are NON-FATAL - caller should continue with fresh build
    /// - Format version mismatches are NON-FATAL - trigger cache rebuild
    /// - Deserialization errors are NON-FATAL - corrupt cache is discarded
    pub fn load(dir: &Path) -> Result<Option<Self>, CacheError> {
        let cache_path = dir.join("incremental.bin");

        // If cache file doesn't exist, return None (cold start)
        if !cache_path.exists() {
            return Ok(None);
        }

        // Read cache file
        let bytes = fs::read(&cache_path)?;

        // Deserialize using bincode
        #[derive(serde::Deserialize)]
        struct SerializedCache {
            format_version: u32,
            rolldown_version: String,
            module_hashes: HashMap<fob_graph::ModuleId, [u8; 32]>,
            graph_bytes: Vec<u8>,
        }

        let cache: SerializedCache = bincode::deserialize(&bytes).map_err(|e| {
            CacheError::DeserializationError(format!(
                "Failed to deserialize incremental cache: {}",
                e
            ))
        })?;

        // Validate format version
        if cache.format_version != Self::FORMAT_VERSION {
            return Err(CacheError::Corrupted(format!(
                "Incompatible incremental cache version: expected {}, got {}",
                Self::FORMAT_VERSION,
                cache.format_version
            )));
        }

        // Deserialize the graph
        let graph = ModuleGraph::from_bytes(&cache.graph_bytes).map_err(|e| {
            CacheError::DeserializationError(format!("Failed to deserialize module graph: {}", e))
        })?;

        Ok(Some(Self {
            graph: Some(graph),
            module_hashes: cache.module_hashes,
            rolldown_version: cache.rolldown_version,
            format_version: cache.format_version,
        }))
    }

    /// Save incremental cache to disk.
    ///
    /// Creates the cache directory if it doesn't exist.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Directory creation fails
    /// - Graph serialization fails
    /// - File write fails
    ///
    /// All errors are NON-FATAL - the build can succeed even if caching fails.
    pub fn save(&self, dir: &Path) -> Result<(), CacheError> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(dir)?;

        let cache_path = dir.join("incremental.bin");

        // Serialize the graph to bytes
        let graph_bytes = if let Some(ref graph) = self.graph {
            graph.to_bytes().map_err(|e| {
                CacheError::SerializationError(format!("Failed to serialize graph: {}", e))
            })?
        } else {
            // No graph to save - this shouldn't happen in practice
            return Err(CacheError::Corrupted(
                "Cannot save incremental cache without a graph".to_string(),
            ));
        };

        // Create serializable structure
        #[derive(serde::Serialize)]
        struct SerializedCache<'a> {
            format_version: u32,
            rolldown_version: &'a str,
            module_hashes: &'a HashMap<fob_graph::ModuleId, [u8; 32]>,
            graph_bytes: &'a [u8],
        }

        let cache = SerializedCache {
            format_version: self.format_version,
            rolldown_version: &self.rolldown_version,
            module_hashes: &self.module_hashes,
            graph_bytes: &graph_bytes,
        };

        // Serialize to bytes using bincode
        let bytes = bincode::serialize(&cache).map_err(|e| {
            CacheError::SerializationError(format!("Failed to encode cache: {}", e))
        })?;

        // Write to file atomically (write to temp file, then rename)
        let temp_path = cache_path.with_extension("tmp");
        fs::write(&temp_path, bytes)?;
        fs::rename(&temp_path, &cache_path)?;

        Ok(())
    }

    /// Check if the cache is valid for the given entry points.
    ///
    /// A cache is valid if:
    /// 1. Rolldown version matches
    /// 2. Entry points match those in the cached graph
    ///
    /// Note: File content hashes are checked separately via ChangeDetector.
    pub fn is_valid_for(&self, entries: &[std::path::PathBuf]) -> bool {
        // Check Rolldown version
        if self.rolldown_version != get_rolldown_version() {
            return false;
        }

        // Check if we have a graph
        let Some(ref graph) = self.graph else {
            return false;
        };

        // Get entry points from the graph
        let Ok(graph_entries) = graph.entry_points() else {
            return false;
        };

        // Convert entry paths to ModuleIds for comparison
        let entry_ids: Vec<_> = entries
            .iter()
            .filter_map(|path| fob_graph::ModuleId::new(path).ok())
            .collect();

        // Check if entry points match
        if entry_ids.len() != graph_entries.len() {
            return false;
        }

        // All entry IDs must be in the graph's entry points
        entry_ids.iter().all(|id| graph_entries.contains(id))
    }
}

impl Default for IncrementalCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the current Rolldown version string.
///
/// This is used for cache invalidation when Rolldown is upgraded.
/// Since we can't easily get the version of a dependency at compile time,
/// we use a hardcoded version that should be updated when Rolldown is upgraded.
///
/// A more robust solution would be to use a build script to extract the version,
/// but for now we rely on the version from workspace dependencies (0.5.1).
fn get_rolldown_version() -> String {
    // Hardcoded version - should match workspace.dependencies.rolldown.version
    // This is good enough for cache invalidation purposes
    "0.5.1".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cache() {
        let cache = IncrementalCache::new();
        assert!(cache.graph.is_none());
        assert!(cache.module_hashes.is_empty());
        assert_eq!(cache.format_version, IncrementalCache::FORMAT_VERSION);
    }

    #[test]
    fn test_is_valid_empty_entries() {
        let cache = IncrementalCache::new();
        assert!(!cache.is_valid_for(&[]));
    }

    #[test]
    fn test_rolldown_version() {
        let version = get_rolldown_version();
        // Should have some version string (compile-time check)
        assert!(!version.is_empty());
    }
}
