#![cfg(feature = "rolldown-integration")]

use crate::graph::{GraphStatistics, ModuleGraph};
use crate::Result;

pub async fn compute_stats(graph: &ModuleGraph) -> Result<GraphStatistics> {
    let unused = graph.unused_exports().await?;
    let unreachable = graph.unreachable_modules().await?;
    let external = graph.external_dependencies().await?;
    let modules = graph.modules().await?;
    let entry_points = graph.entry_points().await?;

    Ok(GraphStatistics::new(
        modules.len(),
        entry_points.len(),
        external.len(),
        modules.iter().filter(|m| m.has_side_effects).count(),
        unused.len(),
        unreachable.len(),
    ))
}
