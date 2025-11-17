use std::path::Path;

use thiserror::Error;

// Note: The standalone analyze API is temporarily disabled as we migrate to the plugin-based
// collection approach. The plugin approach is integrated into the bundler workflow.
// TODO: Implement a standalone analysis path using the plugin approach if needed.

pub mod analyzer;
pub mod cache;
pub mod resolver;
pub mod result;
pub mod stats;
pub mod trace;
pub mod types;
pub mod walker;

#[cfg(test)]
mod tests;

pub use analyzer::Analyzer;
pub use cache::{CacheAnalysis, CacheEffectiveness};
pub use result::{AnalysisResult, AnalyzedBundle};
pub use trace::{ImportOutcome, ImportResolution, RenameEvent, RenamePhase, TransformationTrace};
pub use types::{AnalyzerConfig, ResolveResult};

#[derive(Debug, Error)]
pub enum AnalyzeError {
    #[error("failed to determine current directory: {0}")]
    CurrentDir(#[from] std::io::Error),
    #[error("analysis not implemented: {0}")]
    NotImplemented(String),
}

/// Options for the analyze() function.
#[derive(Clone)]
pub struct AnalyzeOptions {
    /// Framework rules to apply during analysis.
    ///
    /// Joy does not provide any default framework rules. External tools
    /// (like Danny) should provide framework-specific detection logic.
    pub framework_rules: Vec<Box<dyn crate::graph::FrameworkRule>>,

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
/// use fob_core::analysis::{analyze_with_options, AnalyzeOptions};
/// use fob_core::graph::FrameworkRule;
///
/// // Define your own framework rules
/// let options = AnalyzeOptions {
///     framework_rules: vec![Box::new(MyCustomRule)],
/// };
///
/// let result = analyze_with_options(["src/index.tsx"], options).await?;
/// ```
pub async fn analyze_with_options<P>(
    entries: impl IntoIterator<Item = P>,
    _options: AnalyzeOptions,
) -> Result<AnalysisResult, AnalyzeError>
where
    P: AsRef<Path>,
{
    let mut analyzer = Analyzer::new();
    
    // Add entries
    for entry in entries {
        analyzer = analyzer.entry(entry.as_ref());
    }
    
    // Apply framework rules if any (not yet implemented in Analyzer)
    // For now, we'll skip framework rules in standalone analysis
    
    // Apply runtime if available
    // Note: AnalyzeOptions doesn't have runtime, so we'll use default
    // This could be enhanced in the future
    
    analyzer.analyze().await.map_err(|e| {
        AnalyzeError::NotImplemented(format!("Analysis failed: {}", e))
    })
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
/// use fob_core::analysis::analyze;
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
