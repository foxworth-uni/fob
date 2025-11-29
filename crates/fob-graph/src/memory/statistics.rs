//! Statistics methods for ModuleGraph.

use super::super::GraphStatistics;
use super::graph::ModuleGraph;
use crate::Result;

impl ModuleGraph {
    /// Compute statistics snapshot for dashboards.
    pub fn statistics(&self) -> Result<GraphStatistics> {
        let unused = self.unused_exports()?;
        let unreachable = self.unreachable_modules()?;
        let all_modules = self.modules()?;
        let side_effect_module_count = all_modules.iter().filter(|m| m.has_side_effects).count();
        let external_dependency_count = self.external_dependencies()?.len();
        let entry_points = self.entry_points()?;

        Ok(GraphStatistics::new(
            all_modules.len(),
            entry_points.len(),
            external_dependency_count,
            side_effect_module_count,
            unused.len(),
            unreachable.len(),
        ))
    }
}
