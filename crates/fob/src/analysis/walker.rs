//! Graph walker for dependency traversal.
//!
//! Performs BFS traversal of the import graph, parsing modules and building
//! a CollectionState that can be converted to a ModuleGraph.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use path_clean::PathClean;
use rustc_hash::FxHashSet;

use crate::extractors::{extract_scripts, ExtractorError};
use crate::graph::collection::{
    parse_module_structure, CollectedExport, CollectedImport, CollectedImportKind, CollectedModule,
    CollectionState,
};
use crate::runtime::{Runtime, RuntimeError};

use super::resolver::ModuleResolver;
use super::types::{AnalyzerConfig, ResolveResult};

/// Error that can occur during graph walking.
#[derive(Debug, thiserror::Error)]
pub enum WalkerError {
    #[error("Failed to read file '{path}': {source}")]
    ReadFile {
        path: PathBuf,
        #[source]
        source: RuntimeError,
    },

    #[error("Maximum depth exceeded: {depth}")]
    MaxDepthExceeded { depth: usize },

    #[error("Circular dependency detected: {path}")]
    CircularDependency { path: PathBuf },

    #[error("Failed to resolve module '{specifier}' from '{from}': {reason}")]
    ResolutionFailed {
        specifier: String,
        from: PathBuf,
        reason: String,
    },

    #[error("Failed to extract scripts from '{path}': {source}")]
    ExtractionFailed {
        path: PathBuf,
        #[source]
        source: ExtractorError,
    },
}

/// Graph walker that traverses the dependency graph.
pub struct GraphWalker {
    resolver: ModuleResolver,
    config: AnalyzerConfig,
}

impl GraphWalker {
    pub fn new(config: AnalyzerConfig) -> Self {
        let resolver = ModuleResolver::new(config.clone());
        Self { resolver, config }
    }

    /// Walk the dependency graph starting from entry points.
    ///
    /// Returns a CollectionState that can be converted to a ModuleGraph.
    pub async fn walk(&self, runtime: Arc<dyn Runtime>) -> Result<CollectionState, WalkerError> {
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

            // Skip if already visited
            if visited.contains(&current_path) {
                continue;
            }

            // Mark as visited
            visited.insert(current_path.clone());

            // Read the file
            let code = self.read_file(&current_path, runtime.as_ref()).await?;

            // Extract scripts from framework files if needed
            let code_to_parse = self.extract_if_framework(&current_path, &code)?;

            // Parse the module
            let (mut imports, exports, has_side_effects) = self.parse_module(&code_to_parse);

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
            let exports_clone = exports.clone();

            // Resolve imports and populate resolved_path
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

            // Create collected module with resolved imports
            // Store using path relative to cwd for consistency with tests
            let storage_path = self.path_for_storage(&current_path, runtime.as_ref())?;
            let module = CollectedModule {
                id: storage_path.clone(),
                code: Some(code),
                is_entry,
                is_external: false,
                imports,
                exports,
                has_side_effects,
            };

            // Process re-exports (exports with a source) as imports
            for export in &exports_clone {
                if let CollectedExport::All { source } = export {
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

            collection.add_module(storage_path, module);
        }

        Ok(collection)
    }

    /// Read a file from the filesystem.
    async fn read_file(&self, path: &Path, runtime: &dyn Runtime) -> Result<String, WalkerError> {
        let bytes = runtime
            .read_file(path)
            .await
            .map_err(|e| WalkerError::ReadFile {
                path: path.to_path_buf(),
                source: e,
            })?;

        String::from_utf8(bytes).map_err(|e| WalkerError::ReadFile {
            path: path.to_path_buf(),
            source: RuntimeError::Other(format!("Invalid UTF-8: {}", e)),
        })
    }

    /// Extract scripts from framework files if applicable.
    ///
    /// For framework files (.astro, .svelte, .vue), extracts JavaScript/TypeScript
    /// from the component structure. For other files, returns the content as-is.
    fn extract_if_framework(&self, path: &Path, content: &str) -> Result<String, WalkerError> {
        let scripts =
            extract_scripts(path, content).map_err(|e| WalkerError::ExtractionFailed {
                path: path.to_path_buf(),
                source: e,
            })?;

        if scripts.is_empty() {
            // Not a framework file or no scripts found, return as-is
            return Ok(content.to_string());
        }

        // Combine multiple scripts with blank lines (same as plugin behavior)
        let combined: Vec<String> = scripts.iter().map(|s| s.source_text.to_string()).collect();
        Ok(combined.join("\n\n"))
    }

    /// Parse a module to extract imports and exports.
    ///
    /// Uses the existing parse_module_structure function from collection module.
    /// If parsing fails, returns empty imports/exports and assumes side effects.
    fn parse_module(&self, code: &str) -> (Vec<CollectedImport>, Vec<CollectedExport>, bool) {
        parse_module_structure(code).unwrap_or_else(|_| (vec![], vec![], true))
    }

    /// Normalize a path to an absolute path.
    fn normalize_path(&self, path: &Path, runtime: &dyn Runtime) -> Result<PathBuf, WalkerError> {
        let normalized = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let cwd =
                self.resolver
                    .get_cwd(runtime)
                    .map_err(|e| WalkerError::ResolutionFailed {
                        specifier: path.to_string_lossy().to_string(),
                        from: PathBuf::new(),
                        reason: format!("Failed to get cwd: {}", e),
                    })?;
            cwd.join(path)
        };
        // Clean the path to normalize . and .. components
        Ok(normalized.clean())
    }

    /// Convert an absolute path to a path relative to cwd for storage.
    fn path_for_storage(&self, path: &Path, runtime: &dyn Runtime) -> Result<String, WalkerError> {
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
