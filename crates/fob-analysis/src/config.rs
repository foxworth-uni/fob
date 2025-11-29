//! Shared configuration types for analysis and building.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rustc_hash::FxHashMap;

use fob_core::runtime::Runtime;

/// Default maximum depth for graph traversal (DoS protection).
///
/// This prevents infinite loops in circular dependencies and provides
/// a reasonable limit for very deep dependency trees.
pub const DEFAULT_MAX_DEPTH: usize = 1000;

/// Default maximum number of modules to process (DoS protection).
///
/// This prevents processing extremely large codebases that could cause
/// memory exhaustion or excessive processing time.
pub const DEFAULT_MAX_MODULES: usize = 100_000;

/// Maximum depth for graph traversal (DoS protection).
///
/// This newtype ensures type safety when working with depth limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaxDepth(pub usize);

impl MaxDepth {
    /// Create a new MaxDepth value.
    pub fn new(depth: usize) -> Self {
        Self(depth)
    }

    /// Get the depth value.
    pub fn value(&self) -> usize {
        self.0
    }
}

impl Default for MaxDepth {
    /// Default maximum depth.
    fn default() -> Self {
        Self(DEFAULT_MAX_DEPTH)
    }
}

impl From<usize> for MaxDepth {
    fn from(depth: usize) -> Self {
        Self(depth)
    }
}

impl From<MaxDepth> for usize {
    fn from(depth: MaxDepth) -> Self {
        depth.0
    }
}

/// Normalized path with validation.
///
/// This newtype ensures that paths have been validated and normalized,
/// preventing path traversal attacks.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NormalizedPath(pub PathBuf);

impl NormalizedPath {
    /// Create a new NormalizedPath from a PathBuf.
    ///
    /// # Safety
    ///
    /// This method assumes the path has already been validated.
    /// For safe construction, use the validation module.
    pub fn new(path: PathBuf) -> Self {
        Self(path)
    }

    /// Get the path as a reference.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Get the path as PathBuf.
    pub fn into_path_buf(self) -> PathBuf {
        self.0
    }
}

impl AsRef<Path> for NormalizedPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl From<NormalizedPath> for PathBuf {
    fn from(path: NormalizedPath) -> Self {
        path.0
    }
}

/// Maximum file size in bytes (10 MB).
///
/// Files larger than this will be rejected to prevent memory exhaustion
/// and DoS attacks.
pub const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of script tags to process in framework files.
///
/// Framework files (Astro, Svelte, Vue) can contain multiple script blocks.
/// This limit prevents processing files with excessive script tags.
pub const MAX_SCRIPT_TAGS: usize = 100;

/// Configuration options shared between Analyzer and Builder.
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Entry points to analyze.
    pub entries: Vec<PathBuf>,

    /// Packages to treat as external (not analyzed).
    pub external: Vec<String>,

    /// Path aliases for import resolution (e.g., "@" → "./src").
    pub path_aliases: FxHashMap<String, String>,

    /// Whether to follow dynamic imports.
    pub follow_dynamic_imports: bool,

    /// Whether to include TypeScript type-only imports.
    pub include_type_imports: bool,

    /// Maximum depth for graph traversal (DoS protection).
    ///
    /// Default: `DEFAULT_MAX_DEPTH` (1000)
    pub max_depth: Option<usize>,

    /// Maximum number of modules to process (DoS protection).
    ///
    /// Default: `DEFAULT_MAX_MODULES` (100,000)
    pub max_modules: Option<usize>,

    /// Runtime for filesystem operations.
    pub runtime: Option<Arc<dyn Runtime>>,

    /// Current working directory.
    pub cwd: Option<PathBuf>,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            external: Vec::new(),
            path_aliases: FxHashMap::default(),
            follow_dynamic_imports: false,
            include_type_imports: true,
            max_depth: Some(DEFAULT_MAX_DEPTH),
            max_modules: Some(DEFAULT_MAX_MODULES),
            runtime: None,
            cwd: None,
        }
    }
}

/// Result of module resolution.
#[derive(Debug, Clone)]
pub enum ResolveResult {
    /// Module resolved to a local file path.
    Local(PathBuf),

    /// Module is external (npm package, etc.).
    External(String),

    /// Module could not be resolved.
    Unresolved(String),
}

impl ResolveResult {
    pub fn is_local(&self) -> bool {
        matches!(self, ResolveResult::Local(_))
    }

    pub fn is_external(&self) -> bool {
        matches!(self, ResolveResult::External(_))
    }

    pub fn is_unresolved(&self) -> bool {
        matches!(self, ResolveResult::Unresolved(_))
    }
}

impl fmt::Display for ResolveResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveResult::Local(path) => write!(f, "Local({})", path.display()),
            ResolveResult::External(name) => write!(f, "External({})", name),
            ResolveResult::Unresolved(specifier) => write!(f, "Unresolved({})", specifier),
        }
    }
}
