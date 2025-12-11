//! Framework rule methods for ModuleGraph.

use super::super::{Export, ModuleId};
use super::graph::ModuleGraph;
use crate::Result;

impl ModuleGraph {
    /// Apply a custom framework rule.
    ///
    /// Framework rules mark exports as framework-used based on naming conventions.
    /// This prevents false-positive "unused export" warnings.
    ///
    /// Note: FrameworkRule::apply is async, so we use tokio::runtime::Handle::current()
    /// to execute it. This maintains compatibility with the async trait while
    /// Apply a single framework rule asynchronously.
    ///
    /// # Platform Availability
    ///
    /// This method is only available on native platforms (not WASM) because it requires
    /// tokio runtime support.
    #[cfg(not(target_family = "wasm"))]
    pub async fn apply_framework_rule(
        &self,
        rule: Box<dyn super::super::FrameworkRule>,
    ) -> Result<()> {
        rule.apply(self).await
    }

    /// Apply multiple framework rules asynchronously.
    ///
    /// # Platform Availability
    ///
    /// This method is only available on native platforms (not WASM) because it requires
    /// tokio runtime support.
    #[cfg(not(target_family = "wasm"))]
    pub async fn apply_framework_rules(
        &self,
        rules: Vec<Box<dyn super::super::FrameworkRule>>,
    ) -> Result<()> {
        for rule in rules {
            self.apply_framework_rule(rule).await?;
        }
        Ok(())
    }

    /// Check if a direct dependency exists between two modules.
    pub fn has_dependency(&self, from: &ModuleId, to: &ModuleId) -> Result<bool> {
        let deps = self.dependencies(from)?;
        Ok(deps.contains(to))
    }

    /// Get all framework-used exports in the graph.
    pub fn framework_used_exports(&self) -> Result<Vec<(ModuleId, Export)>> {
        let mut result = Vec::new();
        let all_modules = self.modules()?;

        for module in all_modules {
            for export in module.exports.iter() {
                if export.is_framework_used {
                    result.push((module.id.clone(), export.clone()));
                }
            }
        }

        Ok(result)
    }
}
