//! # Analysis Module
//!
//! Graph analysis with I/O and traversal capabilities for JavaScript/TypeScript module graphs.
//!
//! This module provides the `Analyzer` API and related analysis functionality
//! that operates on top of the `fob-graph` data structures. It enables fast,
//! standalone analysis of module dependency graphs without requiring full bundling.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                        Analyzer API                          │
//! │  (Typestate pattern: Unconfigured → Configured → Analysis)  │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      GraphWalker                             │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │  Traversal   │  │    Parser    │  │  Validation  │      │
//! │  │   (BFS)      │→ │  (Extract)   │→ │  (Security)   │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ModuleResolver                            │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
//! │  │  Algorithm   │  │   Aliases    │  │  Extensions  │      │
//! │  │ (Resolution) │→ │  (Path maps) │→ │  (.ts, .js)  │      │
//! │  └──────────────┘  └──────────────┘  └──────────────┘      │
//! └────────────────────┬────────────────────────────────────────┘
//!                      │
//!                      ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    ModuleGraph                              │
//! │              (from fob-graph crate)                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! - **Type-safe API**: Typestate pattern ensures analysis can only be performed
//!   after configuration is complete
//! - **Security**: Path traversal protection and DoS limits (max depth, max modules, file size)
//! - **Framework Support**: Extracts JavaScript/TypeScript from framework components
//! - **Path Aliases**: Supports path alias resolution (e.g., `@` → `./src`)
//! - **External Packages**: Mark npm packages as external to skip analysis
//! - **Usage Analysis**: Compute export usage counts across the module graph
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use fob_graph::{Analyzer, Result};
//!
//! # async fn example() -> Result<()> {
//! // Create analyzer and configure entry points
//! let analysis = Analyzer::new()
//!     .entry("src/index.ts")  // Required: transitions to Configured state
//!     .external(vec!["react", "lodash"])  // Mark as external
//!     .path_alias("@", "./src")  // Configure path aliases
//!     .max_depth(Some(100))  // Set DoS protection limits
//!     .analyze()  // Only available on Configured
//!     .await?;
//!
//! // Use analysis results
//! let unused = analysis.unused_exports()?;
//! println!("Found {} unused exports", unused.len());
//!
//! let circular = analysis.find_circular_dependencies()?;
//! println!("Found {} circular dependencies", circular.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`analyzer`] - Main `Analyzer` API with typestate pattern
//! - [`config`] - Configuration types and constants
//! - [`walker`] - Graph traversal and module parsing
//!   - [`walker::traversal`] - BFS traversal logic
//!   - [`walker::parser`] - Module parsing and script extraction
//!   - [`walker::validation`] - Path security validation
//! - [`resolver`] - Module resolution algorithm
//!   - [`resolver::algorithm`] - Core resolution logic
//!   - [`resolver::aliases`] - Path alias handling
//!   - [`resolver::extensions`] - File extension resolution
//! - [`extractors`] - Framework-specific script extractors
//! - [`result`] - Analysis result types
//!
//! ## Security Considerations
//!
//! The analyzer includes several security features:
//!
//! - **Path Traversal Protection**: All paths are validated to prevent escaping
//!   the current working directory
//! - **DoS Protection**: Limits on maximum depth, module count, and file size
//!   prevent resource exhaustion attacks
//! - **File Size Limits**: Files larger than `MAX_FILE_SIZE` (10MB) are rejected
//!
//! ## Examples
//!
//! See the `examples/` directory for more detailed usage examples:
//!
//! - `basic_analysis.rs` - Simple analysis workflow
//! - `path_aliases.rs` - Configuring and using path aliases
//! - `circular_detection.rs` - Detecting circular dependencies
//! - `framework_components.rs` - Analyzing framework-specific components

use std::path::Path;
use thiserror::Error;

pub mod analyzer;
pub mod cache;
pub mod config;
pub mod extractors;
pub mod resolver;
pub mod result;
pub mod stats;
pub mod trace;
pub mod walker;

#[cfg(test)]
mod tests;

pub use analyzer::{Analyzer, Configured, Unconfigured};
pub use cache::{CacheAnalysis, CacheEffectiveness};
pub use config::{AnalyzerConfig, ResolveResult};
pub use result::AnalysisResult;
pub use trace::{ImportOutcome, ImportResolution, RenameEvent, RenamePhase, TransformationTrace};

/// Error that can occur during analysis.
#[derive(Debug, Error)]
pub enum AnalyzeError {
    /// Failed to determine current working directory.
    #[error("failed to determine current directory: {0}")]
    CurrentDir(#[from] std::io::Error),

    /// Analysis operation failed with a specific reason.
    #[error("analysis failed: {message}")]
    AnalysisFailed {
        /// Human-readable error message describing what went wrong.
        message: String,
        /// Optional context about where the error occurred.
        context: Option<String>,
    },

    /// A specific analysis operation is not yet implemented.
    #[error("analysis not implemented: {0}")]
    NotImplemented(String),
}

impl AnalyzeError {
    /// Create an analysis failed error with a message.
    pub fn analysis_failed(message: impl Into<String>) -> Self {
        Self::AnalysisFailed {
            message: message.into(),
            context: None,
        }
    }

    /// Create an analysis failed error with a message and context.
    pub fn analysis_failed_with_context(
        message: impl Into<String>,
        context: impl Into<String>,
    ) -> Self {
        Self::AnalysisFailed {
            message: message.into(),
            context: Some(context.into()),
        }
    }
}

/// Options for the analyze() function.
#[derive(Clone)]
pub struct AnalyzeOptions {
    /// Framework rules to apply during analysis.
    ///
    /// Joy does not provide any default framework rules. External tools
    /// (like Danny) should provide framework-specific detection logic.
    pub framework_rules: Vec<Box<dyn crate::FrameworkRule>>,

    /// Whether to compute usage counts for exports.
    ///
    /// When enabled, each export will have its `usage_count` field populated
    /// with the number of times it's imported across the module graph.
    ///
    /// Default: true
    pub compute_usage_counts: bool,
}

impl Default for AnalyzeOptions {
    fn default() -> Self {
        Self {
            framework_rules: Vec::new(),
            compute_usage_counts: true,
        }
    }
}

impl std::fmt::Debug for AnalyzeOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnalyzeOptions")
            .field("framework_rules_count", &self.framework_rules.len())
            .finish()
    }
}

/// Analyze module graph with custom options.
///
/// # Example
///
/// ```rust,ignore
/// use super::{analyze_with_options, AnalyzeOptions};
/// use fob_graph::FrameworkRule;
///
/// // Define your own framework rules
/// let options = AnalyzeOptions {
///     framework_rules: vec![Box::new(MyCustomRule)],
///     compute_usage_counts: true,
/// };
///
/// let result = analyze_with_options(["src/index.tsx"], options).await?;
/// ```
pub async fn analyze_with_options<P>(
    entries: impl IntoIterator<Item = P>,
    options: AnalyzeOptions,
) -> Result<AnalysisResult, AnalyzeError>
where
    P: AsRef<Path>,
{
    // Collect entries first
    let entries: Vec<_> = entries.into_iter().collect();
    if entries.is_empty() {
        return Err(AnalyzeError::analysis_failed(
            "At least one entry point is required".to_string(),
        ));
    }

    // Build analyzer with first entry (transitions to Configured)
    let mut analyzer: Analyzer<Configured> = Analyzer::new().entry(entries[0].as_ref());

    // Add remaining entries
    for entry in entries.into_iter().skip(1) {
        analyzer = analyzer.entries([entry.as_ref()]);
    }

    // Analyze with options
    analyzer
        .analyze_with_options(options)
        .await
        .map_err(|e| AnalyzeError::analysis_failed(format!("{}", e)))
}

/// Convenience function using default options.
///
/// This analyzes the module graph without applying any framework rules.
/// For framework-aware analysis, use `analyze_with_options` and provide
/// framework rules explicitly.
///
/// # Example
///
/// ```rust,ignore
/// use super::analyze;
///
/// let result = analyze(["src/index.tsx"]).await?;
/// // No framework rules are applied - pure infrastructure analysis
/// let unused = result.graph.unused_exports();
/// ```
pub async fn analyze<P>(
    entries: impl IntoIterator<Item = P>,
) -> Result<AnalysisResult, AnalyzeError>
where
    P: AsRef<Path>,
{
    analyze_with_options(entries, AnalyzeOptions::default()).await
}
