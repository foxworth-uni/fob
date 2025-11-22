//! Query methods for ModuleGraph.

use std::path::Path;

use super::super::external_dep::ExternalDependency;
use super::super::{Import, Module, ModuleId};
use super::graph::ModuleGraph;
use crate::Result;

impl ModuleGraph {
    /// Retrieve a module by ID.
    ///
    /// Returns an owned `Module`. Internally uses Arc for efficient storage,
    /// so this clone is inexpensive (only shared data like Vec<Import> are reference-counted).
    ///
    /// # Example
    /// ```
    /// # use fob::graph::ModuleGraph;
    /// # fn example(graph: &ModuleGraph, module_id: &fob::graph::ModuleId) -> fob::Result<()> {
    /// let module = graph.module(module_id)?.unwrap();
    /// println!("Path: {}", module.path.display());
    /// # Ok(())
    /// # }
    /// ```
    pub fn module(&self, id: &ModuleId) -> Result<Option<Module>> {
        let inner = self.inner.read();
        Ok(inner.modules.get(id).map(|arc| (**arc).clone()))
    }

    /// Get module by filesystem path.
    ///
    /// Returns an owned `Module`. Internally uses Arc for efficient storage,
    /// so this clone is inexpensive.
    pub fn module_by_path(&self, path: &Path) -> Result<Option<Module>> {
        let inner = self.inner.read();
        Ok(inner
            .modules
            .values()
            .find(|module| module.path == path)
            .map(|arc| (**arc).clone()))
    }

    /// Get all modules.
    ///
    /// Returns owned `Module` instances. Internally uses Arc for efficient storage,
    /// so these clones are inexpensive.
    pub fn modules(&self) -> Result<Vec<Module>> {
        let inner = self.inner.read();
        Ok(inner.modules.values().map(|arc| (**arc).clone()).collect())
    }

    /// Dependencies of a module (forward edges).
    pub fn dependencies(&self, id: &ModuleId) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner
            .dependencies
            .get(id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// Dependents of a module (reverse edges).
    pub fn dependents(&self, id: &ModuleId) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner
            .dependents
            .get(id)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default())
    }

    /// Whether a module is present.
    pub fn contains(&self, id: &ModuleId) -> Result<bool> {
        let inner = self.inner.read();
        Ok(inner.modules.contains_key(id))
    }

    /// Entry points set.
    pub fn entry_points(&self) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner.entry_points.iter().cloned().collect())
    }

    /// Return total module count.
    pub fn len(&self) -> Result<usize> {
        let inner = self.inner.read();
        Ok(inner.modules.len())
    }

    /// Check whether graph is empty.
    pub fn is_empty(&self) -> Result<bool> {
        let inner = self.inner.read();
        Ok(inner.modules.is_empty())
    }

    /// Get imports for a module.
    pub fn imports_for_module(&self, id: &ModuleId) -> Result<Option<Vec<Import>>> {
        let inner = self.inner.read();
        Ok(inner.modules.get(id).map(|m| (*m.imports).clone()))
    }

    /// Aggregate external dependencies based on import data.
    pub fn external_dependencies(&self) -> Result<Vec<ExternalDependency>> {
        let inner = self.inner.read();
        Ok(inner.external_deps.values().cloned().collect())
    }

    /// Compute modules with no dependents and no side effects.
    ///
    /// Returns owned `Module` instances. Internally uses Arc for efficient storage,
    /// so these clones are inexpensive.
    pub fn unreachable_modules(&self) -> Result<Vec<Module>> {
        let inner = self.inner.read();
        let mut unreachable = Vec::new();

        for module in inner.modules.values() {
            if module.is_entry || module.has_side_effects {
                continue;
            }

            let has_dependents = inner
                .dependents
                .get(&module.id)
                .map(|deps| !deps.is_empty())
                .unwrap_or(false);

            if !has_dependents {
                unreachable.push((**module).clone());
            }
        }

        Ok(unreachable)
    }
}
