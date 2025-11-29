use crate::Result;
use crate::{GraphStatistics, ModuleGraph};

pub fn compute_stats(graph: &ModuleGraph) -> Result<GraphStatistics> {
    let unused = graph.unused_exports()?;
    let unreachable = graph.unreachable_modules()?;
    let external = graph.external_dependencies()?;
    let modules = graph.modules()?;
    let entry_points = graph.entry_points()?;

    Ok(GraphStatistics::new(
        modules.len(),
        entry_points.len(),
        external.len(),
        modules.iter().filter(|m| m.has_side_effects).count(),
        unused.len(),
        unreachable.len(),
    ))
}
