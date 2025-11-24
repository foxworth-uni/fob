//! Convert intermediate collection types to final graph representation.
//!
//! This module handles the conversion from `CollectionState` (populated during
//! analysis or bundling) to a fully-formed `ModuleGraph`.
//!
//! # Design Decisions
//!
//! - **Local bindings dropped**: Import specifiers only retain imported names,
//!   not local aliases. This is sufficient for dependency analysis but may
//!   need enhancement for advanced symbol tracking.
//! - **External vs local**: Resolved imports become edges; unresolved become
//!   external dependencies in `graph.external_dependencies()`.
//! - **Error handling**: Uses `CollectionGraphError` to provide context on
//!   conversion failures (e.g., invalid module IDs, graph initialization).

use std::collections::HashMap;
use std::path::PathBuf;

use thiserror::Error;

use super::collection::{
    CollectedExport, CollectedImportKind, CollectedImportSpecifier, CollectedModule,
};
use super::module::ExportsKind as FobExportsKind;
use super::{
    Export, ExportKind, Import, ImportKind, ImportSpecifier as FobImportSpecifier, ModuleId,
    ModuleIdError, SourceSpan,
};

/// Errors that can occur during collection-to-graph conversion
#[derive(Debug, Error)]
pub enum CollectionGraphError {
    /// Failed to initialize the module graph
    #[error("failed to initialize module graph: {0}")]
    GraphInitialization(String),

    /// Module ID conversion failed
    #[error("module id conversion failed for '{path}': {source}")]
    ModuleIdConversion {
        path: String,
        #[source]
        source: ModuleIdError,
    },
}

pub struct PendingImport {
    pub import: Import,
    pub target: Option<String>,
}

// PendingModule is no longer used - conversion happens directly in memory.rs and core.rs

// Helper functions for converting CollectionState to ModuleGraph
// These are used by the in-memory ModuleGraph implementation

pub fn convert_collected_module_id(path: &str) -> Result<ModuleId, CollectionGraphError> {
    let path_buf = PathBuf::from(path);
    ModuleId::new(&path_buf).map_err(|source| CollectionGraphError::ModuleIdConversion {
        path: path.to_string(),
        source,
    })
}

pub fn convert_collected_exports(collected: &CollectedModule, module_id: &ModuleId) -> Vec<Export> {
    let mut exports = Vec::new();

    for export in &collected.exports {
        match export {
            CollectedExport::Named { exported, local: _ } => {
                let kind = if exported == "default" {
                    ExportKind::Default
                } else {
                    ExportKind::Named
                };
                exports.push(Export::new(
                    exported.clone(),
                    kind,
                    false, // is_used
                    false, // is_type_only
                    None,  // re_exported_from
                    false, // is_framework_used
                    false, // came_from_commonjs
                    SourceSpan::new(module_id.as_path(), 0, 0),
                ));
            }
            CollectedExport::Default => {
                exports.push(Export::new(
                    "default".to_string(),
                    ExportKind::Default,
                    false,
                    false,
                    None,
                    false,
                    false,
                    SourceSpan::new(module_id.as_path(), 0, 0),
                ));
            }
            CollectedExport::All { source } => {
                exports.push(Export::new(
                    "*".to_string(),
                    ExportKind::StarReExport,
                    false,
                    false,
                    Some(source.clone()),
                    false,
                    false,
                    SourceSpan::new(module_id.as_path(), 0, 0),
                ));
            }
        }
    }

    exports
}

pub fn convert_collected_imports(
    collected: &CollectedModule,
    module_id: &ModuleId,
    _path_to_id: &HashMap<String, ModuleId>,
) -> Vec<PendingImport> {
    let mut imports = Vec::new();

    for import in &collected.imports {
        let specifiers = import
            .specifiers
            .iter()
            .map(convert_import_specifier)
            .collect();

        let kind = match import.kind {
            CollectedImportKind::Dynamic => ImportKind::Dynamic,
            CollectedImportKind::Static => ImportKind::Static,
            CollectedImportKind::TypeOnly => ImportKind::TypeOnly,
        };

        let fob_import = Import::new(
            import.source.clone(),
            specifiers,
            kind,
            None, // resolved_to will be set later
            SourceSpan::new(module_id.as_path(), 0, 0),
        );

        // Use resolved_path if available (for local modules), otherwise use source (for externals)
        let target = if let Some(ref resolved_path) = import.resolved_path {
            // Local module - use resolved path to look up ModuleId
            Some(resolved_path.clone())
        } else {
            // External or unresolved - use source specifier
            Some(import.source.clone())
        };

        imports.push(PendingImport {
            import: fob_import,
            target,
        });
    }

    imports
}

pub fn convert_import_specifier(spec: &CollectedImportSpecifier) -> FobImportSpecifier {
    match spec {
        CollectedImportSpecifier::Named { imported, local: _ } => {
            FobImportSpecifier::Named(imported.clone())
        }
        CollectedImportSpecifier::Default { local: _ } => FobImportSpecifier::Default,
        CollectedImportSpecifier::Namespace { local: _ } => {
            FobImportSpecifier::Namespace("*".to_string())
        }
    }
}

pub fn infer_exports_kind(exports: &[CollectedExport]) -> FobExportsKind {
    // If we have any exports, assume ESM; otherwise None
    if exports.is_empty() {
        FobExportsKind::None
    } else {
        FobExportsKind::Esm
    }
}

pub fn has_star_export(exports: &[CollectedExport]) -> bool {
    exports
        .iter()
        .any(|e| matches!(e, CollectedExport::All { .. }))
}
