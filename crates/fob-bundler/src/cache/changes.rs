//! Change detection for incremental builds.
//!
//! Detects which modules have changed between builds by comparing content hashes,
//! and computes the transitive set of affected modules using dependency graph traversal.

use blake3::Hasher;
use fob_graph::{ModuleGraph, ModuleId};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use std::fs;
use std::path::Path;

/// Tracks module content hashes for change detection.
///
/// Stores BLAKE3 hashes of file contents to detect modifications between builds.
/// This is used to determine if a cached module graph is still valid or if
/// re-analysis is required.
#[derive(Debug, Clone)]
pub struct ChangeDetector {
    /// BLAKE3 hashes of module file contents, indexed by ModuleId.
    pub module_hashes: HashMap<ModuleId, [u8; 32]>,
}

/// Set of changed modules and their transitive dependents.
///
/// This structure represents the result of change detection, including:
/// - Direct changes (modified, added, removed files)
/// - Transitive changes (modules that depend on modified modules)
#[derive(Debug, Clone)]
pub struct ChangeSet {
    /// Modules that have been modified (content hash changed).
    pub modified: HashSet<ModuleId>,

    /// Modules that are new (not present in previous build).
    pub added: HashSet<ModuleId>,

    /// Modules that were removed (present in previous build but not current).
    pub removed: HashSet<ModuleId>,

    /// All modules affected by changes (includes transitive dependents).
    ///
    /// This is the union of `modified`, `added`, and all modules that
    /// transitively depend on any modified or added module.
    pub affected: HashSet<ModuleId>,
}

impl ChangeSet {
    /// Returns true if there are any changes (modified, added, or removed).
    pub fn has_changes(&self) -> bool {
        !self.modified.is_empty() || !self.added.is_empty() || !self.removed.is_empty()
    }

    /// Returns the total number of affected modules.
    pub fn affected_count(&self) -> usize {
        self.affected.len()
    }
}

impl ChangeDetector {
    /// Create a new empty change detector.
    pub fn new() -> Self {
        Self {
            module_hashes: HashMap::default(),
        }
    }

    /// Create a change detector from an existing module graph.
    ///
    /// Computes BLAKE3 hashes for all modules in the graph by reading their
    /// file contents from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any module file cannot be read
    /// - Module path is invalid
    ///
    /// Note: Errors are non-fatal for incremental caching - if we can't hash
    /// a file, we should fall back to a full rebuild.
    pub fn from_graph(graph: &ModuleGraph) -> crate::Result<Self> {
        let modules = graph.modules()?;
        let mut module_hashes = HashMap::default();

        for module in modules {
            // Skip virtual modules (they have no file on disk)
            if module.id.is_virtual() {
                continue;
            }

            // Hash the file contents
            match hash_file(&module.path) {
                Ok(hash) => {
                    module_hashes.insert(module.id.clone(), hash);
                }
                Err(e) => {
                    // File read errors are non-fatal - we'll just mark this as changed
                    eprintln!(
                        "Warning: Failed to hash file {:?}: {}. Treating as changed.",
                        module.path, e
                    );
                    // Use zero hash to ensure it's detected as changed
                    module_hashes.insert(module.id.clone(), [0u8; 32]);
                }
            }
        }

        Ok(Self { module_hashes })
    }

    /// Detect changes between the cached state and current files.
    ///
    /// Compares the stored hashes against the current file contents to determine
    /// which modules have been modified, added, or removed.
    ///
    /// # Arguments
    ///
    /// * `current_files` - List of (ModuleId, content_hash) pairs for current state
    ///
    /// # Returns
    ///
    /// A `ChangeSet` containing all detected changes. Note that this does NOT
    /// include transitive dependents - use `compute_affected` for that.
    pub fn detect_changes(&self, current_files: &[(ModuleId, [u8; 32])]) -> ChangeSet {
        let mut modified = HashSet::default();
        let mut added = HashSet::default();
        let current_ids: HashSet<ModuleId> =
            current_files.iter().map(|(id, _)| id.clone()).collect();

        // Check for modifications and additions
        for (current_id, current_hash) in current_files {
            match self.module_hashes.get(current_id) {
                Some(cached_hash) => {
                    // Module exists in cache - check if content changed
                    if cached_hash != current_hash {
                        modified.insert(current_id.clone());
                    }
                }
                None => {
                    // Module is new
                    added.insert(current_id.clone());
                }
            }
        }

        // Check for removals
        let removed: HashSet<ModuleId> = self
            .module_hashes
            .keys()
            .filter(|id| !current_ids.contains(id))
            .cloned()
            .collect();

        // Initial affected set is just the direct changes
        // Caller needs to use compute_affected() to get transitive closure
        let mut affected = HashSet::default();
        affected.extend(modified.iter().cloned());
        affected.extend(added.iter().cloned());
        affected.extend(removed.iter().cloned());

        ChangeSet {
            modified,
            added,
            removed,
            affected,
        }
    }

    /// Compute the transitive closure of affected modules.
    ///
    /// Given a set of directly changed modules, compute all modules that
    /// transitively depend on those changes using the dependency graph.
    ///
    /// # Algorithm
    ///
    /// 1. Start with the set of directly changed modules
    /// 2. For each changed module, find all its dependents (reverse edges)
    /// 3. Recursively traverse dependents until no new modules are found
    ///
    /// # Arguments
    ///
    /// * `changed` - Set of directly changed module IDs
    /// * `graph` - The module graph containing dependency information
    ///
    /// # Returns
    ///
    /// A set containing all changed modules plus their transitive dependents.
    pub fn compute_affected(
        &self,
        changed: &HashSet<ModuleId>,
        graph: &ModuleGraph,
    ) -> HashSet<ModuleId> {
        let mut affected = changed.clone();
        let mut to_visit: Vec<ModuleId> = changed.iter().cloned().collect();

        // BFS through reverse edges (dependents)
        while let Some(module_id) = to_visit.pop() {
            // Get all modules that depend on this one
            if let Ok(dependents) = graph.dependents(&module_id) {
                for dependent in dependents {
                    // Only visit each dependent once
                    if affected.insert(dependent.clone()) {
                        to_visit.push(dependent);
                    }
                }
            }
        }

        affected
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute BLAKE3 hash of a file's contents.
///
/// # Errors
///
/// Returns an error if the file cannot be read.
fn hash_file(path: &Path) -> std::io::Result<[u8; 32]> {
    let contents = fs::read(path)?;
    let mut hasher = Hasher::new();
    hasher.update(&contents);
    Ok(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_detector_new() {
        let detector = ChangeDetector::new();
        assert!(detector.module_hashes.is_empty());
    }

    #[test]
    fn test_changeset_has_changes() {
        let empty = ChangeSet {
            modified: HashSet::default(),
            added: HashSet::default(),
            removed: HashSet::default(),
            affected: HashSet::default(),
        };
        assert!(!empty.has_changes());

        let mut with_changes = empty.clone();
        with_changes.modified.insert(ModuleId::new_virtual("test"));
        assert!(with_changes.has_changes());
    }

    #[test]
    fn test_changeset_affected_count() {
        let mut changeset = ChangeSet {
            modified: HashSet::default(),
            added: HashSet::default(),
            removed: HashSet::default(),
            affected: HashSet::default(),
        };

        assert_eq!(changeset.affected_count(), 0);

        changeset.affected.insert(ModuleId::new_virtual("a"));
        changeset.affected.insert(ModuleId::new_virtual("b"));
        assert_eq!(changeset.affected_count(), 2);
    }
}
