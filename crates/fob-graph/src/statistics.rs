use serde::{Deserialize, Serialize};

/// Basic statistics about a `ModuleGraph` useful for dashboards or logging.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphStatistics {
    pub module_count: usize,
    pub entry_point_count: usize,
    pub external_dependency_count: usize,
    pub side_effect_module_count: usize,
    pub unused_export_count: usize,
    pub unreachable_module_count: usize,
}

impl GraphStatistics {
    pub fn new(
        module_count: usize,
        entry_point_count: usize,
        external_dependency_count: usize,
        side_effect_module_count: usize,
        unused_export_count: usize,
        unreachable_module_count: usize,
    ) -> Self {
        Self {
            module_count,
            entry_point_count,
            external_dependency_count,
            side_effect_module_count,
            unused_export_count,
            unreachable_module_count,
        }
    }
}
