//! Persistent build cache for fob-bundler.
//!
//! Provides content-addressed caching of build results to skip Rolldown
//! execution on cache hits. The cache is embedder-controlled - the user
//! specifies where cache files are stored.
//!
//! # Architecture
//!
//! - **Content-addressed**: Cache keys are BLAKE3 hashes of build inputs
//! - **Automatic invalidation**: Key changes when any input changes
//! - **redb backend**: Single database file with ACID transactions
//!
//! # Usage
//!
//! ```rust,no_run
//! use fob_bundler::BuildOptions;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! let result = BuildOptions::new("src/index.ts")
//!     .cache_dir(".cache/fob")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub mod changes;
pub mod incremental;
mod key;
pub mod serialize;
mod storage;

pub use changes::{ChangeDetector, ChangeSet};
pub use incremental::IncrementalCache;
pub use key::CacheKey;
pub use serialize::{CacheMetadata, CachedBuild};
pub use storage::{CacheError, CacheStore};

use std::path::{Path, PathBuf};

/// Configuration for persistent build caching.
///
/// The embedder controls where cache files are stored. Cache keys are
/// content-addressed (BLAKE3 hash of inputs), so invalidation is automatic.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Directory where cache database is stored.
    ///
    /// The cache uses a single redb database file at `<dir>/cache.redb`.
    pub dir: PathBuf,

    /// Force rebuild even if cache exists.
    ///
    /// When true, the cache is bypassed for reads but still written to
    /// after the build completes. Useful for CI or refreshing the cache.
    pub force_rebuild: bool,

    /// Environment variables that affect the cache key.
    ///
    /// If your build output depends on environment variables (e.g., NODE_ENV),
    /// add them here so the cache key changes when they change.
    pub env_vars: Vec<String>,
}

impl CacheConfig {
    /// Create a new cache config with the given directory.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            force_rebuild: false,
            env_vars: Vec::new(),
        }
    }

    /// Set the force rebuild flag.
    pub fn with_force_rebuild(mut self, force: bool) -> Self {
        self.force_rebuild = force;
        self
    }

    /// Add environment variables to include in the cache key.
    pub fn with_env_vars(mut self, vars: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.env_vars = vars.into_iter().map(Into::into).collect();
        self
    }

    /// Check if force rebuild is requested via environment variable.
    pub fn should_force_rebuild(&self) -> bool {
        self.force_rebuild || std::env::var_os("FOB_FORCE_REBUILD").is_some()
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new(".cache/fob")
    }
}

/// Result type for cache operations.
pub type CacheResult<T> = std::result::Result<T, CacheError>;

/// Attempt to load a cached build from the cache store.
///
/// Returns `Ok(cached)` if the cache key exists and is valid,
/// or `Err(CacheError::CacheMiss)` if not found.
pub fn try_load(store: &CacheStore, key: &CacheKey) -> CacheResult<CachedBuild> {
    store.get(key)
}

/// Save a build result to the cache store.
///
/// This is non-fatal - errors are logged but don't fail the build.
pub fn try_save(store: &CacheStore, key: &CacheKey, build: &CachedBuild) -> CacheResult<()> {
    store.put(key, build)
}

/// Compute a cache key for the given build plan.
///
/// The key is a BLAKE3 hash of:
/// 1. Rolldown version
/// 2. Sorted entry paths + content hashes
/// 3. Serialized build options (excluding cache config)
/// 4. Virtual files (sorted path + content hash)
/// 5. Specified environment variables
pub(crate) fn compute_cache_key(
    plan: &crate::builders::common::BundlePlan,
    config: &CacheConfig,
) -> CacheResult<CacheKey> {
    key::compute_cache_key(plan, config)
}

/// Open or create a cache store at the given directory.
///
/// Creates the directory if it doesn't exist.
pub fn open_store(cache_dir: &Path) -> CacheResult<CacheStore> {
    CacheStore::open(cache_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert_eq!(config.dir, PathBuf::from(".cache/fob"));
        assert!(!config.force_rebuild);
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::new("/tmp/cache")
            .with_force_rebuild(true)
            .with_env_vars(["NODE_ENV", "DEBUG"]);

        assert_eq!(config.dir, PathBuf::from("/tmp/cache"));
        assert!(config.force_rebuild);
        assert_eq!(config.env_vars, vec!["NODE_ENV", "DEBUG"]);
    }
}
