//! Shared configuration types for analysis and building.

use rustc_hash::FxHashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::Runtime;

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
    pub max_depth: Option<usize>,
    
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
            max_depth: Some(1000), // Reasonable default to prevent infinite loops
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

