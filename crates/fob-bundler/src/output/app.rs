use rustc_hash::FxHashMap;

use fob_graph::ModuleGraph;

use super::bundle::Bundle;

/// Result of application bundling with optional chunk metadata.
pub struct AppBuild {
    bundle: Bundle,
    chunks: FxHashMap<String, Vec<String>>,
    graph: ModuleGraph,
}

impl AppBuild {
    pub fn bundle(&self) -> &Bundle {
        &self.bundle
    }

    pub fn chunk_manifest(&self) -> &FxHashMap<String, Vec<String>> {
        &self.chunks
    }

    pub fn module_graph(&self) -> &ModuleGraph {
        &self.graph
    }
}
