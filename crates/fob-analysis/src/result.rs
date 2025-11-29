use std::fmt;

use fob_graph::{
    ExternalDependency, GraphStatistics, ModuleGraph, ModuleId, UnusedExport,
    dependency_chain::DependencyChain, symbol::SymbolStatistics,
};

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
    pub fn unused_exports(&self) -> fob_core::Result<Vec<UnusedExport>> {
        self.graph.unused_exports()
    }

    /// Get all external dependencies.
    pub fn external_dependencies(&self) -> fob_core::Result<Vec<ExternalDependency>> {
        self.graph.external_dependencies()
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
    pub fn dependency_chains_to(
        &self,
        target: &ModuleId,
    ) -> fob_core::Result<Vec<DependencyChain>> {
        self.graph.dependency_chains_to(target)
    }

    /// Find circular dependencies in the module graph.
    ///
    /// Returns chains that contain cycles (same module appears multiple times).
    pub fn find_circular_dependencies(&self) -> fob_core::Result<Vec<DependencyChain>> {
        let modules = self.graph.modules()?;
        let mut circular = Vec::new();

        for module in modules {
            let chains = self.graph.dependency_chains_to(&module.id)?;
            for chain in chains {
                if chain.has_cycle() {
                    circular.push(chain);
                }
            }
        }

        Ok(circular)
    }
}

impl fmt::Display for AnalysisResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Analysis Result")?;
        writeln!(f, "================")?;
        writeln!(f, "Entry points: {}", self.entry_points.len())?;

        if let Ok(modules) = self.graph.modules() {
            writeln!(f, "Total modules: {}", modules.len())?;
        }

        if let Ok(unused) = self.unused_exports() {
            writeln!(f, "Unused exports: {}", unused.len())?;
        }

        if let Ok(external) = self.external_dependencies() {
            writeln!(f, "External dependencies: {}", external.len())?;
        }

        writeln!(f, "Warnings: {}", self.warnings.len())?;
        writeln!(f, "Errors: {}", self.errors.len())?;

        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }

        if !self.errors.is_empty() {
            writeln!(f, "\nErrors:")?;
            for error in &self.errors {
                writeln!(f, "  - {}", error)?;
            }
        }

        Ok(())
    }
}
