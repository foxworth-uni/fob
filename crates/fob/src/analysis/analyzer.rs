//! Fast standalone analysis API without bundling.
//!
//! The Analyzer provides a lightweight way to analyze module graphs without
//! the overhead of full bundling. It's ideal for:
//! - IDE integration
//! - CI/CD checks
//! - Documentation generation
//! - Dependency auditing

use std::path::PathBuf;
use std::sync::Arc;

use rustc_hash::FxHashMap;

use crate::analysis::{result::AnalysisResult, stats::compute_stats};
use crate::graph::ModuleGraph;
use crate::runtime::Runtime;
use crate::{Error, Result};

use super::types::AnalyzerConfig;
use super::walker::GraphWalker;

/// Fast standalone analyzer for module graphs.
///
/// # Example
///
/// ```rust,no_run
/// use fob::analysis::analyzer::Analyzer;
///
/// # async fn example() -> fob::Result<()> {
/// let analysis = Analyzer::new()
///     .entry("src/index.ts")
///     .external(vec!["react", "lodash"])
///     .path_alias("@", "./src")
///     .analyze()
///     .await?;
///
/// println!("Unused exports: {}", analysis.unused_exports().await?.len());
/// # Ok(())
/// # }
/// ```
pub struct Analyzer {
    config: AnalyzerConfig,
}

impl Analyzer {
    /// Create a new analyzer with default configuration.
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
        }
    }

    /// Add a single entry point.
    pub fn entry(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.entries.push(path.into());
        self
    }

    /// Add multiple entry points.
    pub fn entries(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.config
            .entries
            .extend(paths.into_iter().map(|p| p.into()));
        self
    }

    /// Mark packages as external (not analyzed).
    pub fn external(mut self, packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.config
            .external
            .extend(packages.into_iter().map(|p| p.into()));
        self
    }

    /// Add a path alias for import resolution.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use fob::analysis::analyzer::Analyzer;
    ///
    /// Analyzer::new()
    ///     .path_alias("@", "./src");
    ///     // Now "@/components/Button" resolves to "./src/components/Button"
    /// ```
    pub fn path_alias(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.config.path_aliases.insert(from.into(), to.into());
        self
    }

    /// Set multiple path aliases at once.
    pub fn path_aliases(mut self, aliases: FxHashMap<String, String>) -> Self {
        self.config.path_aliases = aliases;
        self
    }

    /// Whether to follow dynamic imports (default: false).
    pub fn follow_dynamic_imports(mut self, follow: bool) -> Self {
        self.config.follow_dynamic_imports = follow;
        self
    }

    /// Whether to include TypeScript type-only imports (default: true).
    pub fn include_type_imports(mut self, include: bool) -> Self {
        self.config.include_type_imports = include;
        self
    }

    /// Set maximum depth for graph traversal (DoS protection).
    ///
    /// Default: 1000
    pub fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// Set the runtime for filesystem operations.
    ///
    /// If not set, will attempt to use a default runtime.
    pub fn runtime(mut self, runtime: Arc<dyn Runtime>) -> Self {
        self.config.runtime = Some(runtime);
        self
    }

    /// Set the current working directory.
    pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.config.cwd = Some(cwd.into());
        self
    }

    /// Execute the analysis.
    ///
    /// Returns an `AnalysisResult` containing the module graph and statistics.
    pub async fn analyze(self) -> Result<AnalysisResult> {
        // Validate entries
        if self.config.entries.is_empty() {
            return Err(crate::Error::InvalidConfig(
                "At least one entry point is required".to_string(),
            ));
        }

        // Get or create runtime
        let runtime = self.get_runtime()?;

        // Ensure cwd is set
        let cwd = if self.config.cwd.is_some() {
            self.config.cwd.clone()
        } else {
            runtime.get_cwd().ok()
        };

        let mut config = self.config;
        config.cwd = cwd;

        // Create walker and traverse graph
        let walker = GraphWalker::new(config);
        let collection = walker
            .walk(runtime.clone())
            .await
            .map_err(|e| Error::Operation(format!("Graph walker failed: {}", e)))?;

        // Build module graph from collected data
        let graph = ModuleGraph::from_collected_data(collection)
            .await
            .map_err(|e| Error::Operation(format!("Failed to build module graph: {}", e)))?;

        // Compute statistics
        let stats = compute_stats(&graph).await?;
        let entry_points = graph.entry_points().await?;
        let symbol_stats = graph.symbol_statistics().await?;

        Ok(AnalysisResult {
            graph,
            entry_points,
            warnings: Vec::new(),
            errors: Vec::new(),
            stats,
            symbol_stats,
        })
    }

    /// Get or create a runtime instance.
    fn get_runtime(&self) -> Result<Arc<dyn Runtime>> {
        if let Some(ref runtime) = self.config.runtime {
            Ok(Arc::clone(runtime))
        } else {
            // Try to use default runtime
            #[cfg(not(target_family = "wasm"))]
            {
                use crate::NativeRuntime;
                Ok(Arc::new(NativeRuntime))
            }
            #[cfg(target_family = "wasm")]
            {
                Err(crate::Error::InvalidConfig(
                    "Runtime is required in WASM environment".to_string(),
                ))
            }
        }
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_analyzer_builder() {
        let analyzer = Analyzer::new()
            .entry("src/index.ts")
            .external(vec!["react"])
            .path_alias("@", "./src")
            .max_depth(Some(100));

        assert_eq!(analyzer.config.entries.len(), 1);
        assert_eq!(analyzer.config.external.len(), 1);
        assert_eq!(analyzer.config.path_aliases.len(), 1);
        assert_eq!(analyzer.config.max_depth, Some(100));
    }

    #[tokio::test]
    async fn test_analyzer_requires_entry() {
        let analyzer = Analyzer::new();
        let result = analyzer.analyze().await;

        assert!(result.is_err());
        if let Err(crate::Error::InvalidConfig(msg)) = result {
            assert!(msg.contains("entry"));
        } else {
            panic!("Expected InvalidConfig error");
        }
    }
}
