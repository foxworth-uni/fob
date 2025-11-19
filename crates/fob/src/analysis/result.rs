use crate::graph::{symbol::SymbolStatistics, GraphStatistics, ModuleGraph, ModuleId, ExternalDependency, dependency_chain::DependencyChain};

#[derive(Debug)]
pub struct AnalysisResult {
    pub graph: ModuleGraph,
    pub entry_points: Vec<ModuleId>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub stats: GraphStatistics,
    /// Symbol-level statistics from intra-file dead code analysis
    pub symbol_stats: SymbolStatistics,
}

impl AnalysisResult {
    /// Get all unused exports in the module graph.
    pub async fn unused_exports(&self) -> crate::Result<Vec<crate::graph::UnusedExport>> {
        self.graph.unused_exports().await
    }

    /// Get all external dependencies.
    pub async fn external_dependencies(&self) -> crate::Result<Vec<ExternalDependency>> {
        self.graph.external_dependencies().await
    }

    /// Check if the analysis completed without errors.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if there are any warnings.
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get all dependency chains to a target module.
    ///
    /// Useful for understanding why a module is included or finding circular dependencies.
    pub async fn dependency_chains_to(&self, target: &ModuleId) -> crate::Result<Vec<DependencyChain>> {
        self.graph.dependency_chains_to(target).await
    }

    /// Find circular dependencies in the module graph.
    ///
    /// Returns chains that contain cycles (same module appears multiple times).
    pub async fn find_circular_dependencies(&self) -> crate::Result<Vec<DependencyChain>> {
        let modules = self.graph.modules().await?;
        let mut circular = Vec::new();

        for module in modules {
            let chains = self.graph.dependency_chains_to(&module.id).await?;
            for chain in chains {
                if chain.has_cycle() {
                    circular.push(chain);
                }
            }
        }

        Ok(circular)
    }
}

