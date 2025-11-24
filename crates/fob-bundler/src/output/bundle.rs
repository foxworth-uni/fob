use crate::analysis::AnalyzedBundle;
use crate::BundleOutput;
use fob_analysis::{AnalysisResult, CacheAnalysis, TransformationTrace};
use fob_graph::{GraphStatistics, ModuleGraph, ModuleId};

/// High-level wrapper around `AnalyzedBundle` with ergonomic accessors.
pub struct Bundle {
    inner: AnalyzedBundle,
}

impl From<AnalyzedBundle> for Bundle {
    fn from(inner: AnalyzedBundle) -> Self {
        Self { inner }
    }
}

impl Bundle {
    /// Raw Rolldown bundle output (assets and warnings).
    pub fn output(&self) -> &BundleOutput {
        &self.inner.bundle
    }

    /// Pre-bundling analysis results.
    pub fn analysis(&self) -> &AnalysisResult {
        &self.inner.analysis
    }

    /// Dependency graph captured during analysis.
    pub fn module_graph(&self) -> &ModuleGraph {
        &self.inner.analysis.graph
    }

    /// Entry points analysed for this bundle.
    pub fn entry_points(&self) -> &[ModuleId] {
        &self.inner.analysis.entry_points
    }

    /// Aggregate statistics for the analysed module graph.
    pub fn stats(&self) -> &GraphStatistics {
        &self.inner.analysis.stats
    }

    /// Cache metrics gathered during bundling.
    pub fn cache(&self) -> &CacheAnalysis {
        &self.inner.cache
    }

    /// Transformation trace if `JOY_TRACE=1`.
    pub fn trace(&self) -> Option<&TransformationTrace> {
        self.inner.trace.as_ref()
    }

    /// Warnings discovered during static analysis.
    pub fn analysis_warnings(&self) -> &[String] {
        &self.inner.analysis.warnings
    }

    /// Errors discovered during static analysis.
    pub fn analysis_errors(&self) -> &[String] {
        &self.inner.analysis.errors
    }

    /// Consume the bundle and return the underlying analysed payload.
    pub fn into_inner(self) -> AnalyzedBundle {
        self.inner
    }
}
