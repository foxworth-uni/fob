//! Dependency chain analysis methods for ModuleGraph.

use rustc_hash::FxHashMap as HashMap;

use super::super::ModuleId;
use super::super::dependency_chain::{ChainAnalysis, DependencyChain, find_chains};
use super::graph::ModuleGraph;
use fob_core::Result;

impl ModuleGraph {
    /// Find all dependency chains from entry points to a target module.
    ///
    /// This traces all possible paths through the import graph, useful for understanding
    /// why a module is included in the bundle and what depends on it.
    pub fn dependency_chains_to(&self, target: &ModuleId) -> Result<Vec<DependencyChain>> {
        let entry_points = self.entry_points()?;
        let inner = self.inner.read();

        let get_deps = |module: &ModuleId| -> Vec<ModuleId> {
            inner
                .dependencies
                .get(module)
                .map(|set| set.iter().cloned().collect())
                .unwrap_or_default()
        };

        Ok(find_chains(&entry_points, target, get_deps))
    }

    /// Analyze dependency chains to a module.
    ///
    /// Provides comprehensive statistics about all paths leading to a module.
    pub fn analyze_dependency_chains(&self, target: &ModuleId) -> Result<ChainAnalysis> {
        let chains = self.dependency_chains_to(target)?;
        Ok(ChainAnalysis::from_chains(target.clone(), chains))
    }

    /// Get the import depth of a module from entry points.
    ///
    /// Returns the shortest distance from any entry point to this module,
    /// or None if the module is unreachable.
    pub fn import_depth(&self, module: &ModuleId) -> Result<Option<usize>> {
        let analysis = self.analyze_dependency_chains(module)?;
        Ok(analysis.min_depth)
    }

    /// Group modules by their import depth from entry points.
    ///
    /// This creates layers of the dependency graph, useful for visualizing
    /// the structure and understanding module organization.
    pub fn modules_by_depth(&self) -> Result<HashMap<usize, Vec<ModuleId>>> {
        let all_modules = self.modules()?;
        let mut by_depth: HashMap<usize, Vec<ModuleId>> = HashMap::default();

        for module in all_modules {
            if let Some(depth) = self.import_depth(&module.id)? {
                by_depth.entry(depth).or_default().push(module.id.clone());
            }
        }

        Ok(by_depth)
    }

    /// Check if a module is only reachable through dead code.
    ///
    /// A module is considered "reachable only through dead code" if:
    /// - It has no direct path from any entry point, OR
    /// - All paths to it go through unreachable modules
    ///
    /// This is a strong indicator that the module can be safely removed.
    pub fn is_reachable_only_through_dead_code(&self, module: &ModuleId) -> Result<bool> {
        let analysis = self.analyze_dependency_chains(module)?;

        // If not reachable at all, it's definitely dead
        if !analysis.is_reachable() {
            return Ok(true);
        }

        // If we have any chain, the module is reachable from an entry point
        // More sophisticated analysis would check if all chains go through
        // modules that are themselves unreachable, but that requires
        // recursive analysis which we'll skip for now.
        Ok(false)
    }
}
