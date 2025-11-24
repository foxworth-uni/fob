//! BFS traversal logic for graph walking.
//!
//! This module contains the breadth-first search algorithm that traverses
//! the dependency graph starting from entry points.

use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;

use rustc_hash::FxHashSet;

use fob_graph::collection::{CollectedImportKind, CollectionState};
use fob::runtime::Runtime;

use crate::config::{AnalyzerConfig, ResolveResult};
use crate::resolver::ModuleResolver;
use super::parser::ModuleParser;
use super::validation::normalize_and_validate_path;
use super::WalkerError;

/// BFS traversal state and logic.
pub struct Traversal<'a> {
    resolver: &'a ModuleResolver,
    config: &'a AnalyzerConfig,
    parser: ModuleParser,
}

impl<'a> Traversal<'a> {
    pub fn new(resolver: &'a ModuleResolver, config: &'a AnalyzerConfig) -> Self {
        Self {
            resolver,
            config,
            parser: ModuleParser,
        }
    }

    /// Perform BFS traversal of the dependency graph.
    ///
    /// This is the main traversal loop that processes modules in breadth-first order,
    /// resolving imports and adding new modules to the queue as they're discovered.
    pub async fn traverse(
        &self,
        runtime: Arc<dyn Runtime>,
    ) -> Result<CollectionState, WalkerError> {
        let mut collection = CollectionState::new();
        let mut visited = FxHashSet::default();
        let mut queue = VecDeque::new();
        let mut depth_map: HashMap<PathBuf, usize> = HashMap::new();

        // Initialize queue with entry points
        for entry in &self.config.entries {
            let entry_path = self.normalize_path(entry, runtime.as_ref())?;
            queue.push_back((entry_path.clone(), 0));
            depth_map.insert(entry_path.clone(), 0);
            let entry_storage_path = self.path_for_storage(&entry_path, runtime.as_ref())?;
            collection.mark_entry(entry_storage_path);
        }

        // BFS traversal
        while let Some((current_path, depth)) = queue.pop_front() {
            // Check max depth
            if let Some(max_depth) = self.config.max_depth {
                if depth > max_depth {
                    return Err(WalkerError::MaxDepthExceeded { depth });
                }
            }

            // Check max modules
            if let Some(max_modules) = self.config.max_modules {
                if visited.len() >= max_modules {
                    return Err(WalkerError::TooManyModules {
                        count: visited.len(),
                        max: max_modules,
                    });
                }
            }

            // Skip if already visited
            if visited.contains(&current_path) {
                continue;
            }

            // Mark as visited
            visited.insert(current_path.clone());

            // Process the module
            let module = self
                .parser
                .process_module(&current_path, runtime.as_ref())
                .await?;

            // Determine if this is an entry point
            let is_entry = self.config.entries.iter().any(|e| {
                self.normalize_path(e, runtime.as_ref())
                    .map(|p| p == current_path)
                    .unwrap_or(false)
            });

            // Mark entry point in collection using relative path
            if is_entry {
                let entry_storage_path = self.path_for_storage(&current_path, runtime.as_ref())?;
                collection.mark_entry(entry_storage_path);
            }

            // Clone exports before moving into module (needed for re-export processing)
            let exports_clone = module.exports.clone();

            // Resolve imports and populate resolved_path
            let mut imports = module.imports;
            for import in &mut imports {
                // Skip dynamic imports if not following them
                if import.kind == CollectedImportKind::Dynamic
                    && !self.config.follow_dynamic_imports
                {
                    continue;
                }

                // Resolve the import
                let resolve_result = self
                    .resolver
                    .resolve(&import.source, &current_path, runtime.as_ref())
                    .await
                    .map_err(|e| WalkerError::ResolutionFailed {
                        specifier: import.source.clone(),
                        from: current_path.clone(),
                        reason: e.to_string(),
                    })?;

                match resolve_result {
                    ResolveResult::Local(resolved_path) => {
                        let normalized = self.normalize_path(&resolved_path, runtime.as_ref())?;

                        // Store resolved path for graph building
                        import.resolved_path =
                            Some(self.path_for_storage(&normalized, runtime.as_ref())?);

                        // Check for circular dependency
                        if visited.contains(&normalized) {
                            // Already processed, skip
                            continue;
                        }

                        // Add to queue if not already queued
                        if !depth_map.contains_key(&normalized) {
                            let new_depth = depth + 1;
                            depth_map.insert(normalized.clone(), new_depth);
                            queue.push_back((normalized, new_depth));
                        }
                    }
                    ResolveResult::External(_) => {
                        // External dependency - resolved_path stays None
                        // It will be handled when building the graph
                    }
                    ResolveResult::Unresolved(_) => {
                        // Could not resolve - resolved_path stays None
                        // This allows analysis to continue even with missing modules
                    }
                }
            }

            // Process re-exports (exports with a source) as imports
            for export in &exports_clone {
                if let fob_graph::collection::CollectedExport::All { source } = export {
                    // Re-export - treat as import to follow the dependency
                    // Skip dynamic imports if not following them (re-exports are never dynamic)

                    // Resolve the re-export source
                    if let Ok(ResolveResult::Local(resolved_path)) = self
                        .resolver
                        .resolve(source, &current_path, runtime.as_ref())
                        .await
                    {
                        let normalized = self.normalize_path(&resolved_path, runtime.as_ref())?;

                        // Check for circular dependency
                        if visited.contains(&normalized) {
                            // Already processed, skip
                            continue;
                        }

                        // Add to queue if not already queued
                        if !depth_map.contains_key(&normalized) {
                            let new_depth = depth + 1;
                            depth_map.insert(normalized.clone(), new_depth);
                            queue.push_back((normalized, new_depth));
                        }
                    }
                }
            }

            // Create collected module with resolved imports
            // Store using path relative to cwd for consistency with tests
            let storage_path = self.path_for_storage(&current_path, runtime.as_ref())?;
            let collected_module = fob_graph::collection::CollectedModule {
                id: storage_path.clone(),
                code: Some(module.code),
                is_entry,
                is_external: false,
                imports,
                exports: module.exports,
                has_side_effects: module.has_side_effects,
            };

            collection.add_module(storage_path, collected_module);
        }

        Ok(collection)
    }

    /// Normalize a path to an absolute path with security validation.
    fn normalize_path(&self, path: &std::path::Path, runtime: &dyn Runtime) -> Result<PathBuf, WalkerError> {
        let cwd = self
            .resolver
            .get_cwd(runtime)
            .map_err(|e| WalkerError::ResolutionFailed {
                specifier: path.to_string_lossy().to_string(),
                from: PathBuf::new(),
                reason: format!("Failed to get cwd: {}", e),
            })?;

        normalize_and_validate_path(path, &cwd).map_err(|e| WalkerError::PathTraversal {
            path: e.path,
            cwd: e.cwd,
        })
    }

    /// Convert an absolute path to a path relative to cwd for storage.
    fn path_for_storage(&self, path: &std::path::Path, runtime: &dyn Runtime) -> Result<String, WalkerError> {
        let cwd = self
            .resolver
            .get_cwd(runtime)
            .map_err(|e| WalkerError::ResolutionFailed {
                specifier: path.to_string_lossy().to_string(),
                from: PathBuf::new(),
                reason: format!("Failed to get cwd: {}", e),
            })?;

        // Try to make it relative to cwd
        if let Ok(rel) = path.strip_prefix(&cwd) {
            Ok(rel.to_string_lossy().to_string())
        } else {
            // If it's not under cwd, use the absolute path as string
            Ok(path.to_string_lossy().to_string())
        }
    }
}

