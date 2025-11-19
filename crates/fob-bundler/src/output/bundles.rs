use rustc_hash::FxHashMap;

use fob::graph::ModuleGraph;

use super::{bundle::Bundle, import_map::ImportMap};

/// Aggregated result for component bundling.
pub struct ComponentBuild {
    bundles: FxHashMap<String, Bundle>,
    shared_graph: ModuleGraph,
    shared_imports: Vec<String>,
    import_map: ImportMap,
}

impl ComponentBuild {
    pub fn len(&self) -> usize {
        self.bundles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bundles.is_empty()
    }

    pub fn get(&self, entry: &str) -> Option<&Bundle> {
        self.bundles.get(entry)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Bundle)> {
        self.bundles.iter()
    }

    pub fn shared_graph(&self) -> &ModuleGraph {
        &self.shared_graph
    }

    pub fn shared_imports(&self) -> &[String] {
        &self.shared_imports
    }

    pub fn import_map(&self) -> &ImportMap {
        &self.import_map
    }
}
