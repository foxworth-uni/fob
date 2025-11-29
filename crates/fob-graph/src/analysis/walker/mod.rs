//! Graph walker for dependency traversal.
//!
//! Performs BFS traversal of the import graph, parsing modules and building
//! a CollectionState that can be converted to a ModuleGraph.

mod parser;
mod traversal;
mod validation;

pub use validation::{PathTraversalError, normalize_and_validate_path, validate_path_within_cwd};

use std::sync::Arc;

use crate::collection::CollectionState;
use crate::runtime::Runtime;

use crate::analysis::config::AnalyzerConfig;
use crate::analysis::resolver::ModuleResolver;

/// Error that can occur during graph walking.
#[derive(Debug, thiserror::Error)]
pub enum WalkerError {
    #[error("Failed to read file '{path}': {source}")]
    ReadFile {
        path: std::path::PathBuf,
        #[source]
        source: crate::runtime::RuntimeError,
    },

    #[error("Maximum depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },

    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: std::path::PathBuf },

    #[error("Failed to resolve module '{specifier}' from '{from}': {reason}")]
    ResolutionFailed {
        specifier: String,
        from: std::path::PathBuf,
        reason: String,
    },

    #[error("Failed to extract scripts from '{path}': {source}")]
    ExtractionFailed {
        path: std::path::PathBuf,
        #[source]
        source: crate::analysis::extractors::ExtractorError,
    },

    #[error("Path traversal detected: path '{path}' escapes from cwd '{cwd}'")]
    PathTraversal {
        path: std::path::PathBuf,
        cwd: std::path::PathBuf,
    },

    #[error("Too many modules processed: {count} modules (max: {max} allowed)")]
    TooManyModules { count: usize, max: usize },

    #[error("File too large: {path} is {size} bytes (max: {max} bytes)")]
    FileTooLarge {
        path: std::path::PathBuf,
        size: usize,
        max: usize,
    },
}

/// Graph walker that traverses the dependency graph.
pub struct GraphWalker {
    resolver: ModuleResolver,
    config: AnalyzerConfig,
}

impl GraphWalker {
    /// Create a new graph walker with the given configuration.
    pub fn new(config: AnalyzerConfig) -> Self {
        let resolver = ModuleResolver::new(config.clone());
        Self { resolver, config }
    }

    /// Walk the dependency graph starting from entry points.
    ///
    /// Returns a CollectionState that can be converted to a ModuleGraph.
    pub async fn walk(&self, runtime: Arc<dyn Runtime>) -> Result<CollectionState, WalkerError> {
        let traversal = traversal::Traversal::new(&self.resolver, &self.config);
        traversal.traverse(runtime).await
    }
}
