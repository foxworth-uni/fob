use std::collections::HashMap;
use std::path::PathBuf;

use rustc_hash::FxHashMap;
use thiserror::Error;

use super::module_collection_plugin::{
    CollectedExport, CollectedModule, CollectionState, ImportSpecifier,
};
use super::semantic::analyze_symbols;
use super::{
    Export, ExportKind, ExternalDependency, Import, ImportKind, ImportSpecifier as FobImportSpecifier,
    Module, ModuleGraph, ModuleId, ModuleIdError, SourceSpan, SourceType,
};
use super::module::{ExportsKind as FobExportsKind, ModuleFormat as FobModuleFormat};

#[derive(Debug, Error)]
pub enum CollectionGraphError {
    #[error("module id conversion failed for '{path}': {source}")]
    ModuleIdConversion {
        path: String,
        #[source]
        source: ModuleIdError,
    },
}

struct PendingImport {
    import: Import,
    target: Option<String>,
}

struct PendingModule {
    id: ModuleId,
    module: Module,
    imports: Vec<PendingImport>,
}

impl ModuleGraph {
    /// Create a ModuleGraph from collected module data
    pub async fn from_collected_data(
        collection: CollectionState,
    ) -> Result<Self, CollectionGraphError> {
        let graph = ModuleGraph::new().await.map_err(|_| {
            CollectionGraphError::ModuleIdConversion {
                path: "graph initialization".to_string(),
                source: ModuleIdError::EmptyPath,
            }
        })?;

        let mut path_to_id: HashMap<String, ModuleId> = HashMap::new();
        let mut pending_modules: Vec<PendingModule> = Vec::new();
        let mut external_names: Vec<String> = Vec::new();

        // First pass: create module IDs and identify externals
        for (path, collected) in &collection.modules {
            if collected.is_external {
                external_names.push(path.clone());
                continue;
            }

            let module_id = convert_collected_module_id(path)?;
            path_to_id.insert(path.clone(), module_id);
        }

        // Second pass: convert modules
        for (path, collected) in &collection.modules {
            if collected.is_external {
                continue;
            }

            let module_id = path_to_id
                .get(path)
                .ok_or_else(|| CollectionGraphError::ModuleIdConversion {
                    path: path.clone(),
                    source: ModuleIdError::EmptyPath,
                })?;

            let exports = convert_collected_exports(collected, module_id);
            let imports = convert_collected_imports(collected, module_id, &path_to_id);

            let has_side_effects = collected.has_side_effects;

            // Perform semantic analysis to extract symbols
            let source_type = SourceType::from_path(module_id.as_path());
            let code = collected.code.as_deref().unwrap_or("");
            let mut symbol_table = analyze_symbols(
                code,
                module_id.as_path().to_str().unwrap_or("unknown"),
                source_type,
            )
            .unwrap_or_default();

            // Link exports to symbols - mark symbols as exported
            let export_names: Vec<String> = exports.iter().map(|e| e.name.clone()).collect();
            symbol_table.mark_exports(&export_names);

            let mut builder = Module::builder(
                module_id.clone(),
                module_id.as_path().to_path_buf(),
                source_type,
            )
            .exports(exports)
            .side_effects(has_side_effects)
            .original_size(code.len())
            .bundled_size(None)
            .external(false)
            .symbol_table(symbol_table)
            .module_format(FobModuleFormat::Unknown) // We don't have this info from collection
            .exports_kind(infer_exports_kind(&collected.exports))
            .has_star_exports(has_star_export(&collected.exports))
            .execution_order(None); // We don't have execution order from collection

            if collected.is_entry {
                builder = builder.entry(true);
            }

            let pending = PendingModule {
                id: module_id.clone(),
                module: builder.build(),
                imports,
            };
            pending_modules.push(pending);
        }

        let mut external_aggregate: FxHashMap<String, ExternalDependency> = FxHashMap::default();

        // Third pass: resolve imports and build graph
        for pending in pending_modules {
            let module_id = pending.id.clone();
            let mut module = pending.module;

            let mut resolved_imports = Vec::with_capacity(pending.imports.len());
            for mut pending_import in pending.imports {
                if let Some(target_path) = pending_import.target {
                    if let Some(target_id) = path_to_id.get(&target_path) {
                        pending_import.import.resolved_to = Some(target_id.clone());
                        graph
                            .add_dependency(module_id.clone(), target_id.clone())
                            .await
                            .map_err(|_e| CollectionGraphError::ModuleIdConversion {
                                path: format!("dependency {} -> {}", module_id, target_id),
                                source: ModuleIdError::EmptyPath,
                            })?;
                    } else {
                        // Any unresolved import is treated as external dependency
                        // This includes both explicitly marked externals and package imports
                        let dep = external_aggregate
                            .entry(target_path.clone())
                            .or_insert_with(|| ExternalDependency::new(target_path.clone()));
                        dep.push_importer(module_id.clone());
                    }
                }
                resolved_imports.push(pending_import.import);
            }

            // Update module with resolved imports before adding
            module.imports = resolved_imports;
            graph
                .add_module(module)
                .await
                .map_err(|_e| CollectionGraphError::ModuleIdConversion {
                    path: module_id.to_string(),
                    source: ModuleIdError::EmptyPath,
                })?;
        }

        for dep in external_aggregate.into_values() {
            let specifier = dep.specifier.clone();
            graph
                .add_external_dependency(dep)
                .await
                .map_err(|_| CollectionGraphError::ModuleIdConversion {
                    path: specifier,
                    source: ModuleIdError::EmptyPath,
                })?;
        }

        Ok(graph)
    }
}

fn convert_collected_module_id(path: &str) -> Result<ModuleId, CollectionGraphError> {
    let path_buf = PathBuf::from(path);
    ModuleId::new(&path_buf).map_err(|source| {
        CollectionGraphError::ModuleIdConversion {
            path: path.to_string(),
            source,
        }
    })
}

fn convert_collected_exports(
    collected: &CollectedModule,
    module_id: &ModuleId,
) -> Vec<Export> {
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

fn convert_collected_imports(
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

        let kind = if import.is_dynamic {
            ImportKind::Dynamic
        } else {
            ImportKind::Static
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

fn convert_import_specifier(spec: &ImportSpecifier) -> FobImportSpecifier {
    match spec {
        ImportSpecifier::Named { imported, local: _ } => {
            FobImportSpecifier::Named(imported.clone())
        }
        ImportSpecifier::Default { local: _ } => FobImportSpecifier::Default,
        ImportSpecifier::Namespace { local: _ } => FobImportSpecifier::Namespace("*".to_string()),
    }
}

fn infer_exports_kind(exports: &[CollectedExport]) -> FobExportsKind {
    // If we have any exports, assume ESM; otherwise None
    if exports.is_empty() {
        FobExportsKind::None
    } else {
        FobExportsKind::Esm
    }
}

fn has_star_export(exports: &[CollectedExport]) -> bool {
    exports.iter().any(|e| matches!(e, CollectedExport::All { .. }))
}
