//! Mutation methods for ModuleGraph.

use std::sync::Arc;

use super::super::Module;
use super::super::external_dep::ExternalDependency;
use super::graph::ModuleGraph;
use crate::Result;

impl ModuleGraph {
    /// Add a module into the graph.
    pub fn add_module(&self, module: Module) -> Result<()> {
        // Prepare external dependency data before acquiring the lock
        let mut external_deps_to_add = Vec::new();
        for import in module.imports.iter() {
            if import.resolved_to.is_none() && !import.source.is_empty() {
                external_deps_to_add.push((import.source.clone(), module.id.clone()));
            }
        }

        // Now acquire the write lock and perform all updates
        let mut inner = self.inner.write();

        // Track if it's an entry point
        if module.is_entry {
            inner.entry_points.insert(module.id.clone());
        }

        // Add external dependencies
        for (source, importer_id) in external_deps_to_add {
            let dep = inner
                .external_deps
                .entry(source.clone())
                .or_insert_with(|| ExternalDependency::new(source));
            dep.push_importer(importer_id);
        }

        // Store the module wrapped in Arc for cheap cloning
        inner.modules.insert(module.id.clone(), Arc::new(module));

        Ok(())
    }

    /// Add a dependency edge, creating forward and reverse mappings.
    pub fn add_dependency(
        &self,
        from: super::super::ModuleId,
        to: super::super::ModuleId,
    ) -> Result<()> {
        let mut inner = self.inner.write();

        // Add forward edge (HashSet prevents duplicates)
        inner
            .dependencies
            .entry(from.clone())
            .or_default()
            .insert(to.clone());

        // Add reverse edge (HashSet prevents duplicates)
        inner.dependents.entry(to).or_default().insert(from);

        Ok(())
    }

    /// Add multiple dependencies from a single module.
    pub fn add_dependencies<I>(&self, from: super::super::ModuleId, targets: I) -> Result<()>
    where
        I: IntoIterator<Item = super::super::ModuleId>,
    {
        for target in targets {
            self.add_dependency(from.clone(), target)?;
        }
        Ok(())
    }

    /// Mark a module as an entry point.
    pub fn add_entry_point(&self, id: super::super::ModuleId) -> Result<()> {
        let mut inner = self.inner.write();
        inner.entry_points.insert(id.clone());

        // Update the module itself if it exists (requires cloning from Arc to modify)
        if let Some(module_arc) = inner.modules.get(&id) {
            let mut module = (**module_arc).clone();
            module.is_entry = true;
            inner.modules.insert(id, Arc::new(module));
        }

        Ok(())
    }

    /// Add an external dependency record.
    pub fn add_external_dependency(&self, dep: ExternalDependency) -> Result<()> {
        let mut inner = self.inner.write();
        inner.external_deps.insert(dep.specifier.clone(), dep);
        Ok(())
    }
}
