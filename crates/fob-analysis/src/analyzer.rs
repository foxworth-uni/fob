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

use crate::{AnalyzeOptions, result::AnalysisResult, stats::compute_stats};
use fob_core::runtime::Runtime;
use fob_core::{Error, Result};
use fob_graph::ModuleGraph;

use super::config::AnalyzerConfig;
use super::walker::GraphWalker;

/// Typestate marker for an unconfigured analyzer (no entry points yet).
#[derive(Debug, Clone, Copy)]
pub struct Unconfigured;

/// Typestate marker for a configured analyzer (has entry points).
#[derive(Debug, Clone, Copy)]
pub struct Configured;

/// Fast standalone analyzer for module graphs.
///
/// Uses the typestate pattern to ensure that analysis can only be performed
/// after at least one entry point has been configured. This prevents runtime
/// errors and makes the API more type-safe.
///
/// # Example
///
/// ```rust,no_run
/// use fob_analysis::analyzer::Analyzer;
///
/// # async fn example() -> fob_core::Result<()> {
/// let analysis = Analyzer::new()
///     .entry("src/index.ts")  // Transitions to Configured state
///     .external(vec!["react", "lodash"])
///     .path_alias("@", "./src")
///     .analyze()  // Only available on Configured
///     .await?;
///
/// println!("Unused exports: {}", analysis.unused_exports()?.len());
/// # Ok(())
/// # }
/// ```
pub struct Analyzer<State = Unconfigured> {
    config: AnalyzerConfig,
    _state: std::marker::PhantomData<State>,
}

impl Analyzer<Unconfigured> {
    /// Create a new analyzer with default configuration.
    ///
    /// Returns an analyzer in the `Unconfigured` state. You must call `entry()`
    /// before you can call `analyze()`.
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
            _state: std::marker::PhantomData,
        }
    }

    /// Add a single entry point.
    ///
    /// This transitions the analyzer to the `Configured` state, allowing
    /// `analyze()` to be called.
    pub fn entry(mut self, path: impl Into<PathBuf>) -> Analyzer<Configured> {
        self.config.entries.push(path.into());
        Analyzer {
            config: self.config,
            _state: std::marker::PhantomData,
        }
    }
}

impl<State> Analyzer<State> {
    /// Add multiple entry points.
    ///
    /// This method is available in both `Unconfigured` and `Configured` states.
    /// If called on `Unconfigured`, it transitions to `Configured`.
    pub fn entries(
        mut self,
        paths: impl IntoIterator<Item = impl Into<PathBuf>>,
    ) -> Analyzer<Configured> {
        self.config
            .entries
            .extend(paths.into_iter().map(|p| p.into()));
        Analyzer {
            config: self.config,
            _state: std::marker::PhantomData,
        }
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
    /// use fob_analysis::analyzer::Analyzer;
    ///
    /// Analyzer::new()
    ///     .entry("src/index.ts")
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

    /// Set maximum number of modules to process (DoS protection).
    ///
    /// Default: 100,000
    pub fn max_modules(mut self, modules: Option<usize>) -> Self {
        self.config.max_modules = modules;
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
}

impl Analyzer<Configured> {
    /// Execute the analysis with default options.
    ///
    /// Returns an `AnalysisResult` containing the module graph and statistics.
    ///
    /// This method is only available on `Analyzer<Configured>`, ensuring
    /// that at least one entry point has been set.
    pub async fn analyze(self) -> Result<AnalysisResult> {
        self.analyze_with_options(AnalyzeOptions::default()).await
    }

    /// Execute the analysis with custom options.
    ///
    /// This method allows you to specify framework rules and control whether
    /// usage counts are computed.
    ///
    /// # Arguments
    ///
    /// * `options` - Analysis options including framework rules and usage count settings
    ///
    /// # Returns
    ///
    /// An `AnalysisResult` containing the module graph and statistics.
    ///
    /// This method is only available on `Analyzer<Configured>`, ensuring
    /// that at least one entry point has been set.
    pub async fn analyze_with_options(self, options: AnalyzeOptions) -> Result<AnalysisResult> {
        // Entries are guaranteed to exist by the typestate

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
            .map_err(|e| Error::Operation(format!("Failed to build module graph: {}", e)))?;

        // Apply framework rules if provided
        if !options.framework_rules.is_empty() {
            #[cfg(not(target_family = "wasm"))]
            {
                graph
                    .apply_framework_rules(options.framework_rules)
                    .map_err(|e| {
                        Error::Operation(format!("Failed to apply framework rules: {}", e))
                    })?;
            }
            #[cfg(target_family = "wasm")]
            {
                // Framework rules require tokio runtime which isn't available in WASM
                // Silently skip them in WASM environments
            }
        }

        // Compute usage counts if requested
        if options.compute_usage_counts {
            graph
                .compute_export_usage_counts()
                .map_err(|e| Error::Operation(format!("Failed to compute usage counts: {}", e)))?;
        }

        // Compute statistics
        let stats = compute_stats(&graph)?;
        let entry_points = graph.entry_points()?;
        let symbol_stats = graph.symbol_statistics()?;

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
                use fob_core::NativeRuntime;
                Ok(Arc::new(NativeRuntime))
            }
            #[cfg(target_family = "wasm")]
            {
                Err(fob_core::Error::InvalidConfig(
                    "Runtime is required in WASM environment".to_string(),
                ))
            }
        }
    }
}

impl Default for Analyzer<Unconfigured> {
    fn default() -> Self {
        Self::new()
    }
}

// Type alias for backward compatibility
/// Type alias for the default analyzer state (unconfigured).
///
/// For new code, prefer explicitly using `Analyzer<Unconfigured>` or
/// `Analyzer<Configured>` to make the state clear.
pub type AnalyzerDefault = Analyzer<Unconfigured>;

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
    async fn test_analyzer_typestate() {
        // Unconfigured analyzer cannot call analyze()
        let _unconfigured: Analyzer<Unconfigured> = Analyzer::new();
        // This would be a compile error:
        // let _ = unconfigured.analyze().await;

        // Configured analyzer can call analyze()
        let configured: Analyzer<Configured> = Analyzer::new().entry("src/index.ts");
        // This compiles (though it will fail at runtime without proper setup)
        let _result = configured.analyze().await;
    }

    #[tokio::test]
    async fn test_analyzer_entries_transition() {
        // entries() transitions from Unconfigured to Configured
        let configured: Analyzer<Configured> = Analyzer::new().entries(vec!["src/index.ts"]);
        let _result = configured.analyze().await;
    }
}
