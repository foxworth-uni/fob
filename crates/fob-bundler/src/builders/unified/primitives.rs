//! Composable primitives for configuring bundler behavior.
//!
//! This module provides three orthogonal axes for controlling the bundler:
//!
//! 1. [`EntryMode`] - Do entries share code? (Shared vs Isolated)
//! 2. [`CodeSplittingConfig`] - Code splitting configuration (Option = on/off)
//! 3. [`ExternalConfig`] - External dependencies (None, List, FromManifest)
//! 4. [`IncrementalConfig`] - Incremental module graph caching (optional)
//!
//! These primitives can be composed explicitly to achieve any valid build configuration.

use std::path::PathBuf;

/// Controls whether entry points share code or are isolated.
///
/// This determines whether multiple entry points share a single bundle context
/// (allowing code splitting and shared chunks) or produce completely independent bundles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryMode {
    /// Entries can share chunks (was: Unified).
    ///
    /// Entry points share a single output context and can reference shared chunks.
    /// This is the only valid option for single-entry builds, and required when
    /// code splitting is enabled.
    ///
    /// Returns [`BuildOutput::Single`](super::BuildOutput::Single).
    #[default]
    Shared,

    /// Each entry stands alone (was: Separate).
    ///
    /// No code sharing between bundles; each is self-contained. This is useful
    /// for component libraries where each component should be independently
    /// consumable without requiring shared runtime chunks.
    ///
    /// Requires multiple entry points and is incompatible with code splitting.
    ///
    /// Returns [`BuildOutput::Multiple`](super::BuildOutput::Multiple).
    Isolated,
}

/// Configuration for code splitting.
///
/// Code splitting extracts shared dependencies into separate chunks that can be
/// loaded on-demand or preloaded, reducing initial bundle size and improving
/// caching efficiency.
///
/// When `None`, code splitting is disabled. When `Some(config)`, code splitting
/// is enabled with the specified thresholds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSplittingConfig {
    /// Minimum chunk size in bytes (e.g., 20000 = 20KB).
    pub min_size: u32,

    /// Minimum number of entry points that must import the same module (minimum: 2).
    ///
    /// This was previously called `min_share_count` but `min_imports` is clearer
    /// about what's being counted.
    pub min_imports: u32,
}

/// Controls external dependency handling.
///
/// This determines which dependencies (from node_modules) are externalized
/// (not bundled) vs included in the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum ExternalConfig {
    /// Bundle everything (was: BundlingMode::Full).
    ///
    /// All imports, including from node_modules, are included in the bundle.
    /// This produces self-contained output that can run without external dependencies.
    #[default]
    None,

    /// Externalize specific packages (explicit list).
    ///
    /// Only the packages in this list are marked as external.
    /// All other dependencies are bundled.
    List(Vec<String>),

    /// Externalize dependencies from package.json manifest.
    ///
    /// Reads the `dependencies` and `peerDependencies` fields from the
    /// specified package.json file and externalizes those packages.
    FromManifest(std::path::PathBuf),
}

impl CodeSplittingConfig {
    /// Create code splitting config with custom thresholds.
    #[inline]
    pub fn new(min_size: u32, min_imports: u32) -> Self {
        Self {
            min_size,
            min_imports,
        }
    }

    /// Convert to Rolldown's AdvancedChunksOptions.
    pub(crate) fn to_rolldown_options(&self) -> rolldown::AdvancedChunksOptions {
        rolldown::AdvancedChunksOptions {
            min_size: Some(self.min_size as f64),
            min_share_count: Some(self.min_imports),
            max_size: None,
            min_module_size: None,
            max_module_size: None,
            include_dependencies_recursively: None,
            groups: Some(vec![]),
        }
    }
}

impl Default for CodeSplittingConfig {
    fn default() -> Self {
        Self {
            min_size: 20_000,
            min_imports: 2,
        }
    }
}

impl std::fmt::Display for EntryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryMode::Shared => write!(f, "shared"),
            EntryMode::Isolated => write!(f, "isolated"),
        }
    }
}

impl std::fmt::Display for CodeSplittingConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "code_splitting(min_size={}, min_imports={})",
            self.min_size, self.min_imports
        )
    }
}

impl std::fmt::Display for ExternalConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalConfig::None => write!(f, "none"),
            ExternalConfig::List(packages) => {
                write!(f, "list({})", packages.join(", "))
            }
            ExternalConfig::FromManifest(path) => {
                write!(f, "from_manifest({})", path.display())
            }
        }
    }
}

/// Configuration for incremental module graph caching.
///
/// When enabled, the module graph is cached between builds and only
/// reprocessed when files change. This can significantly speed up
/// watch mode and incremental rebuilds.
///
/// # Cache Strategy
///
/// The cache stores:
/// - Serialized module graph (post-Rolldown analysis)
/// - BLAKE3 hashes of all module file contents
/// - Rolldown version string (for invalidation on upgrades)
///
/// On subsequent builds:
/// 1. Load cached graph and hashes
/// 2. Compute current file hashes
/// 3. If no changes detected, reuse cached graph
/// 4. Otherwise, run full analysis and update cache
///
/// # Limitations
///
/// Rolldown still runs every time (it's a black box we can't cache).
/// We only cache the post-Rolldown graph analysis phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalConfig {
    /// Directory where incremental cache is stored.
    ///
    /// The cache uses a single binary file at `<cache_dir>/incremental.bin`.
    /// Parent directories are created automatically if they don't exist.
    pub cache_dir: PathBuf,
}

impl IncrementalConfig {
    /// Create incremental config with the given cache directory.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fob_bundler::IncrementalConfig;
    ///
    /// let config = IncrementalConfig::new(".fob-cache");
    /// ```
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self::new(".fob-cache")
    }
}

impl std::fmt::Display for IncrementalConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "incremental({})", self.cache_dir.display())
    }
}
