//! Traversal methods for ModuleGraph.

use std::collections::VecDeque;

use rustc_hash::FxHashSet as HashSet;

use super::graph::ModuleGraph;
use super::super::ModuleId;
use crate::Result;

impl ModuleGraph {
    /// Returns true if `from` depends on `to` (directly or transitively).
    pub fn depends_on(&self, from: &ModuleId, to: &ModuleId) -> Result<bool> {
        if from == to {
            return Ok(true);
        }

        let inner = self.inner.read();
        let mut visited = HashSet::default();
        let mut queue = VecDeque::new();
        queue.push_back(from.clone());

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }

            if let Some(deps) = inner.dependencies.get(&current) {
                for dep in deps {
                    if dep == to {
                        return Ok(true);
                    }
                    queue.push_back(dep.clone());
                }
            }
        }

        Ok(false)
    }

    /// Collect transitive dependencies of a module.
    pub fn transitive_dependencies(&self, id: &ModuleId) -> Result<HashSet<ModuleId>> {
        let mut visited = HashSet::default();
        let mut queue = VecDeque::new();
        queue.push_back(id.clone());

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }

            let deps = self.dependencies(&current)?;
            for next in deps {
                if !visited.contains(&next) {
                    queue.push_back(next);
                }
            }
        }

        visited.remove(id);
        Ok(visited)
    }
}

