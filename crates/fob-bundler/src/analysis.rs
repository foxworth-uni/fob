//! Bundler-specific analysis types.

use std::sync::Arc;

use fob_analysis::{AnalysisResult, CacheAnalysis, TransformationTrace};

use crate::builders::asset_registry::AssetRegistry;
use crate::BundleOutput;

/// Analysis result combined with bundle output.
pub struct AnalyzedBundle {
    pub bundle: BundleOutput,
    pub analysis: AnalysisResult,
    pub cache: CacheAnalysis,
    pub trace: Option<TransformationTrace>,
    /// Asset registry containing discovered static assets
    pub asset_registry: Option<Arc<AssetRegistry>>,
}
