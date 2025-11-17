//! Bundle metadata extraction for export/import analysis.
//!
//! This module provides rich metadata about bundle contents by analyzing the entire
//! module graph. Unlike simpler approaches that only look at entry points, this
//! extracts information from ALL modules in the bundle.
//!
//! # Why Extract from All Modules?
//!
//! Entry points are just the beginning of the dependency tree. A bundle contains many
//! modules that aren't entry points but still export functionality that might be
//! re-exported or used dynamically. By analyzing the full graph, we can:
//!
//! - Detect all public exports (including re-exports)
//! - Track all external dependencies (not just top-level imports)
//! - Calculate accurate size metrics
//! - Identify default exports anywhere in the bundle
//!
//! # Performance Considerations
//!
//! Metadata extraction is lazy - it only happens when explicitly requested via
//! `Bundle::metadata()`. The results should be cached by the caller if needed
//! repeatedly.

use serde::{Deserialize, Serialize};

use crate::graph::{ExportKind, ModuleGraph};
use crate::Result;

/// Comprehensive metadata about a bundle's contents.
///
/// This aggregates information across all modules in the bundle to provide
/// a complete picture of what the bundle exports and imports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// All named exports from the bundle (aggregated from all modules).
    exports: Vec<ExportInfo>,

    /// All external imports required by the bundle (aggregated from all modules).
    imports: Vec<ImportInfo>,

    /// Total size of all assets in bytes.
    total_size: usize,

    /// Number of assets in the bundle.
    asset_count: usize,

    /// Number of modules analyzed.
    module_count: usize,
}

/// Information about a single export from the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportInfo {
    /// The export name (or "default" for default exports).
    pub name: String,

    /// Whether this is a default export.
    pub is_default: bool,

    /// The module that exports this (path or module ID).
    pub source_module: String,

    /// Whether the export is used within the bundle.
    pub is_used: bool,
}

/// Information about an external import required by the bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportInfo {
    /// The import specifier (e.g., "react", "lodash/debounce").
    pub specifier: String,

    /// Modules that import this dependency.
    pub imported_by: Vec<String>,

    /// Whether this import is type-only (won't exist at runtime).
    pub is_type_only: bool,
}

impl BundleMetadata {
    /// Extracts metadata from a module graph.
    ///
    /// This analyzes ALL modules in the graph, not just entry points, to build
    /// a complete picture of the bundle's exports and dependencies.
    ///
    /// # Arguments
    ///
    /// * `graph` - The module graph to analyze
    /// * `total_size` - Total size of all assets in bytes
    /// * `asset_count` - Number of assets in the bundle
    pub async fn from_graph(graph: &ModuleGraph, total_size: usize, asset_count: usize) -> Result<Self> {
        let exports = extract_exports(graph).await?;
        let imports = extract_imports(graph).await?;
        let module_count = graph.len().await?;

        Ok(Self {
            exports,
            imports,
            total_size,
            asset_count,
            module_count,
        })
    }

    /// Returns all exports from the bundle.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::output::metadata::BundleMetadata;
    /// # fn example(metadata: &BundleMetadata) {
    /// for export in metadata.exports() {
    ///     println!("Export: {} from {}", export.name, export.source_module);
    /// }
    /// # }
    /// ```
    pub fn exports(&self) -> &[ExportInfo] {
        &self.exports
    }

    /// Returns all external imports required by the bundle.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::output::metadata::BundleMetadata;
    /// # fn example(metadata: &BundleMetadata) {
    /// for import in metadata.imports() {
    ///     println!("Import: {} (used by {} modules)",
    ///         import.specifier,
    ///         import.imported_by.len()
    ///     );
    /// }
    /// # }
    /// ```
    pub fn imports(&self) -> &[ImportInfo] {
        &self.imports
    }

    /// Returns the total size of all assets in bytes.
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// Returns the number of assets in the bundle.
    pub fn asset_count(&self) -> usize {
        self.asset_count
    }

    /// Returns the number of modules analyzed.
    pub fn module_count(&self) -> usize {
        self.module_count
    }

    /// Checks if the bundle has a specific named export.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::output::metadata::BundleMetadata;
    /// # fn example(metadata: &BundleMetadata) {
    /// if metadata.has_export("MyComponent") {
    ///     println!("Bundle exports MyComponent");
    /// }
    /// # }
    /// ```
    pub fn has_export(&self, name: &str) -> bool {
        self.exports.iter().any(|e| e.name == name)
    }

    /// Checks if the bundle has a default export.
    ///
    /// This is useful for determining how to import the bundle:
    /// - Default export: `import Bundle from './bundle.js'`
    /// - Named exports only: `import { Component } from './bundle.js'`
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::output::metadata::BundleMetadata;
    /// # fn example(metadata: &BundleMetadata) {
    /// if metadata.has_default_export() {
    ///     println!("import Bundle from './bundle.js'");
    /// } else {
    ///     println!("import {{ Component }} from './bundle.js'");
    /// }
    /// # }
    /// ```
    pub fn has_default_export(&self) -> bool {
        self.exports.iter().any(|e| e.is_default)
    }

    /// Returns all default exports in the bundle.
    ///
    /// Note: A bundle can have multiple default exports from different modules.
    /// The bundler typically hoists one of these as the bundle's default export.
    pub fn default_exports(&self) -> Vec<&ExportInfo> {
        self.exports.iter().filter(|e| e.is_default).collect()
    }

    /// Returns all named exports (non-default) in the bundle.
    pub fn named_exports(&self) -> Vec<&ExportInfo> {
        self.exports.iter().filter(|e| !e.is_default).collect()
    }

    /// Returns unused exports that could potentially be tree-shaken.
    ///
    /// These are exports that aren't imported by any module in the bundle.
    /// They might still be part of the public API if they're exported from
    /// entry points.
    pub fn unused_exports(&self) -> Vec<&ExportInfo> {
        self.exports.iter().filter(|e| !e.is_used).collect()
    }
}

/// Extracts all exports from the module graph.
///
/// This walks through every module (not just entry points) and collects all exports.
/// This is important because:
/// - Intermediate modules might re-export things
/// - Non-entry modules might still have public exports
/// - We want a complete picture of what the bundle contains
async fn extract_exports(graph: &ModuleGraph) -> Result<Vec<ExportInfo>> {
    let mut exports = Vec::new();
    let modules = graph.modules().await?;

    for module in modules {
        for export in &module.exports {
            exports.push(ExportInfo {
                name: if matches!(export.kind, ExportKind::Default) {
                    "default".to_string()
                } else {
                    export.name.clone()
                },
                is_default: matches!(export.kind, ExportKind::Default),
                source_module: module.path.display().to_string(),
                is_used: export.is_used,
            });
        }
    }

    Ok(exports)
}

/// Extracts all external imports from the module graph.
///
/// This identifies all external dependencies (packages from npm, etc.) that the
/// bundle requires at runtime. We aggregate imports by specifier to show which
/// modules depend on each external package.
async fn extract_imports(graph: &ModuleGraph) -> Result<Vec<ImportInfo>> {
    use rustc_hash::FxHashMap;

    let mut import_map: FxHashMap<String, ImportInfo> = FxHashMap::default();
    let modules = graph.modules().await?;

    for module in modules {
        for import in &module.imports {
            // Only collect external imports (npm packages, etc.)
            // Skip relative imports as they're bundled
            if import.is_external() {
                let info = import_map
                    .entry(import.source.clone())
                    .or_insert_with(|| ImportInfo {
                        specifier: import.source.clone(),
                        imported_by: Vec::new(),
                        is_type_only: import.is_type_only(),
                    });

                let module_path = module.path.display().to_string();
                if !info.imported_by.contains(&module_path) {
                    info.imported_by.push(module_path);
                }

                // If any import is runtime, mark the whole dependency as runtime
                if !import.is_type_only() {
                    info.is_type_only = false;
                }
            }
        }
    }

    Ok(import_map.into_values().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{
        Export, ExportKind, Import, ImportKind, ImportSpecifier, Module, ModuleGraph, ModuleId,
        SourceSpan, SourceType,
    };

    async fn create_test_graph() -> ModuleGraph {
        let graph = ModuleGraph::new().await.unwrap();

        // Entry module with exports
        let entry_id = ModuleId::new_virtual("entry.js");
        let entry = Module::builder(entry_id.clone(), "entry.js".into(), SourceType::JavaScript)
            .exports(vec![
                Export::new(
                    "Component",
                    ExportKind::Named,
                    true,
                    false,
                    None,
                    false,
                    false,
                    SourceSpan::new("entry.js", 0, 0),
                ),
                Export::new(
                    "default",
                    ExportKind::Default,
                    true,
                    false,
                    None,
                    false,
                    false,
                    SourceSpan::new("entry.js", 0, 0),
                ),
            ])
            .imports(vec![Import::new(
                "react",
                vec![ImportSpecifier::Named("useState".into())],
                ImportKind::Static,
                None,
                SourceSpan::new("entry.js", 0, 0),
            )])
            .entry(true)
            .build();

        graph.add_module(entry).await.unwrap();
        graph
    }

    #[tokio::test]
    async fn test_extract_exports() {
        let graph = create_test_graph().await;
        let exports = extract_exports(&graph).await.unwrap();

        assert_eq!(exports.len(), 2);
        assert!(exports.iter().any(|e| e.name == "Component"));
        assert!(exports.iter().any(|e| e.is_default));
    }

    #[tokio::test]
    async fn test_extract_imports() {
        let graph = create_test_graph().await;
        let imports = extract_imports(&graph).await.unwrap();

        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].specifier, "react");
        assert!(!imports[0].is_type_only);
    }

    #[tokio::test]
    async fn test_has_default_export() {
        let graph = create_test_graph().await;
        let metadata = BundleMetadata::from_graph(&graph, 1024, 1).await.unwrap();

        assert!(metadata.has_default_export());
    }

    #[tokio::test]
    async fn test_has_export() {
        let graph = create_test_graph().await;
        let metadata = BundleMetadata::from_graph(&graph, 1024, 1).await.unwrap();

        assert!(metadata.has_export("Component"));
        assert!(!metadata.has_export("NonExistent"));
    }

    #[tokio::test]
    async fn test_named_exports() {
        let graph = create_test_graph().await;
        let metadata = BundleMetadata::from_graph(&graph, 1024, 1).await.unwrap();

        let named = metadata.named_exports();
        assert_eq!(named.len(), 1);
        assert_eq!(named[0].name, "Component");
    }
}
