use fob_core::analysis::{AnalysisResult, TransformationTrace};
use fob_core::graph::{GraphStatistics, ModuleGraph};
use fob_core::output::Bundle as LibraryBuild;
use fob_core::CacheAnalysis;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisDocument {
    pub stats: GraphStatistics,
    pub graph: ModuleGraph,
    pub cache: CacheAnalysis,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub trace: Option<TransformationTrace>,
}

impl AnalysisDocument {
    pub fn from_library_bundle(bundle: &LibraryBuild) -> Self {
        Self {
            stats: bundle.stats().clone(),
            graph: bundle.module_graph().clone(),
            cache: bundle.cache().clone(),
            warnings: bundle.analysis_warnings().to_vec(),
            errors: bundle.analysis_errors().to_vec(),
            trace: bundle.trace().cloned(),
        }
    }

    pub fn from_analysis(analysis: &AnalysisResult, cache: CacheAnalysis) -> Self {
        Self {
            stats: analysis.stats.clone(),
            graph: analysis.graph.clone(),
            cache,
            warnings: analysis.warnings.clone(),
            errors: analysis.errors.clone(),
            trace: None,
        }
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}
