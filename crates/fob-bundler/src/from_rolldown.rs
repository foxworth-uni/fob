use std::collections::HashMap;

use rolldown_common::{
    EntryPoint, EntryPointKind, ExportsKind as RdExportsKind, ImportKind as RdImportKind,
    ImportRecordIdx, ImportRecordMeta, Module as RdModule, ModuleDefFormat as RdModuleDefFormat,
    ModuleIdx, ModuleTable, NormalModule, Specifier,
};
use rustc_hash::FxHashMap;
use thiserror::Error;

use fob::graph::module::{ExportsKind as FobExportsKind, ModuleFormat as FobModuleFormat};
use fob::graph::semantic::analyze_symbols;
use fob::graph::{
    Export, ExportKind, ExternalDependency, Import, ImportKind, ImportSpecifier, Module,
    ModuleGraph, ModuleId, ModuleIdError, SourceSpan, SourceType,
};

#[derive(Debug, Error)]
pub enum RolldownGraphError {
    #[error("module id conversion failed for '{path}': {source}")]
    ModuleIdConversion {
        path: String,
        #[source]
        source: ModuleIdError,
    },
}

struct PendingImport {
    import: Import,
    target: Option<ModuleIdx>,
}

struct PendingModule {
    id: ModuleId,
    module: Module,
    imports: Vec<PendingImport>,
}

/// Convert Rolldown module table and entry points to a ModuleGraph.
pub async fn from_rolldown_parts(
    module_table: &ModuleTable,
    entry_points: &[EntryPoint],
) -> Result<ModuleGraph, RolldownGraphError> {
    let graph = ModuleGraph::new().await.map_err(|_| RolldownGraphError::ModuleIdConversion {
        path: "graph initialization".to_string(),
        source: ModuleIdError::EmptyPath,
    })?;
    let mut idx_to_id: HashMap<ModuleIdx, ModuleId> = HashMap::new();
    let mut external_idx_to_name: HashMap<ModuleIdx, String> = HashMap::new();
    let mut pending_modules: Vec<PendingModule> = Vec::new();

    let entry_idx: FxHashMap<ModuleIdx, EntryPointKind> = entry_points
        .iter()
        .map(|entry| (entry.idx, entry.kind))
        .collect();

    for (idx, module) in module_table.modules.iter_enumerated() {
        match module {
            RdModule::Normal(normal) => {
                let module_id = convert_module_id(normal)?;
                idx_to_id.insert(idx, module_id.clone());

                let exports = convert_exports(normal, &module_id);
                let imports = convert_imports(normal, &module_id);
                let has_side_effects = normal.ecma_view.side_effects.has_side_effects();

                // Perform semantic analysis to extract symbols
                let source_type = SourceType::from_path(module_id.as_path());
                let mut symbol_table = analyze_symbols(
                    &normal.ecma_view.source,
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
                .original_size(normal.ecma_view.source.len())
                .bundled_size(None)
                .external(false)
                .symbol_table(symbol_table)
                .module_format(convert_module_format(normal.ecma_view.def_format))
                .exports_kind(convert_exports_kind(normal.ecma_view.exports_kind))
                .has_star_exports(normal.ecma_view.meta.has_star_export())
                .execution_order(Some(normal.exec_order));

                if normal.is_user_defined_entry
                    || entry_idx
                        .get(&idx)
                        .map(|k| k.is_user_defined())
                        .unwrap_or(false)
                {
                    builder = builder.entry(true);
                }

                let pending = PendingModule {
                    id: module_id,
                    module: builder.build(),
                    imports,
                };
                pending_modules.push(pending);
            }
            RdModule::External(external) => {
                external_idx_to_name.insert(external.idx, external.id.to_string());
            }
        }
    }

    let mut external_aggregate: FxHashMap<String, ExternalDependency> = FxHashMap::default();

    for pending in pending_modules {
        let module_id = pending.id.clone();
        let mut module = pending.module;
        
        let mut resolved_imports = Vec::with_capacity(pending.imports.len());
        for mut pending_import in pending.imports {
            if let Some(target_idx) = pending_import.target {
                if let Some(target_id) = idx_to_id.get(&target_idx) {
                    pending_import.import.resolved_to = Some(target_id.clone());
                    graph.add_dependency(module_id.clone(), target_id.clone()).await
                        .map_err(|_e| RolldownGraphError::ModuleIdConversion {
                            path: format!("dependency {} -> {}", module_id, target_id),
                            source: ModuleIdError::EmptyPath,
                        })?;
                } else if let Some(specifier) = external_idx_to_name.get(&target_idx) {
                    let dep = external_aggregate
                        .entry(specifier.clone())
                        .or_insert_with(|| ExternalDependency::new(specifier.clone()));
                    dep.push_importer(module_id.clone());
                }
            }
            resolved_imports.push(pending_import.import);
        }

        // Update module with resolved imports before adding
        module.imports = resolved_imports;
        graph.add_module(module).await
            .map_err(|_e| RolldownGraphError::ModuleIdConversion {
                path: module_id.to_string(),
                source: ModuleIdError::EmptyPath,
            })?;
    }

    for dep in external_aggregate.into_values() {
        let specifier = dep.specifier.clone();
        graph.add_external_dependency(dep).await
            .map_err(|_| RolldownGraphError::ModuleIdConversion {
                path: specifier,
                source: ModuleIdError::EmptyPath,
            })?;
    }

    Ok(graph)
}

fn convert_module_id(normal: &NormalModule) -> Result<ModuleId, RolldownGraphError> {
    // Convert rolldown ModuleId to fob ModuleId
    let raw = normal.id.as_ref();
    if raw.starts_with('\0') || raw.starts_with("rolldown:") {
        Ok(ModuleId::new_virtual(raw.to_string()))
    } else {
        ModuleId::new(raw).map_err(|source| RolldownGraphError::ModuleIdConversion {
            path: normal.id.to_string(),
            source,
        })
    }
}

fn convert_exports(normal: &NormalModule, module_id: &ModuleId) -> Vec<Export> {
    let mut exports = Vec::new();
    
    // Build a map of import record indices to their module sources
    // This helps us detect re-exports by matching exports to imports
    let mut import_record_sources: FxHashMap<ImportRecordIdx, String> = FxHashMap::default();
    for (idx, record) in normal.ecma_view.import_records.iter_enumerated() {
        import_record_sources.insert(idx, record.module_request.to_string());
    }
    
    // Build a map of export names to their import record indices (if re-exported)
    let mut export_to_import_record: FxHashMap<String, ImportRecordIdx> = FxHashMap::default();
    for named_import in normal.ecma_view.named_imports.values() {
        if let Specifier::Literal(lit) = &named_import.imported {
            // If this imported name matches an export name, it's likely a re-export
            export_to_import_record.insert(lit.to_string(), named_import.record_id);
        }
    }
    
    for (name, export) in &normal.ecma_view.named_exports {
        let span = export.span;
        
        // Determine export kind
        let (kind, re_exported_from) = if name.as_str() == "default" {
            (ExportKind::Default, None)
        } else if let Some(record_idx) = export_to_import_record.get(name.as_str()) {
            // This export matches an import, so it's likely a re-export
            let source = import_record_sources.get(record_idx).cloned();
            (ExportKind::ReExport, source)
        } else {
            (ExportKind::Named, None)
        };
        
        exports.push(Export::new(
            name.to_string(),
            kind,
            false,  // is_used
            false,  // is_type_only
            re_exported_from,
            false,  // is_framework_used
            export.came_from_commonjs,  // ⬅️ NEW: Use rolldown's CJS flag
            SourceSpan::new(module_id.as_path(), span.start, span.end),
        ));
    }

    // ⬅️ NEW: Add star re-exports (export * from './module')
    // These don't appear in named_exports, only in import_records with IsExportStar flag
    for (_idx, record) in normal.ecma_view.import_records.iter_enumerated() {
        if record.meta.contains(ImportRecordMeta::IsExportStar) {
            // ImportRecord doesn't have span info, so we use a zero span
            // The span info is in the import lookup table, but star re-exports may not be there
            exports.push(Export::new(
                "*".to_string(),  // Star re-export uses "*" as the name
                ExportKind::StarReExport,  // ⬅️ NEW: Use new enum variant
                false,  // is_used
                false,  // is_type_only
                Some(record.module_request.to_string()),  // Source module
                false,  // is_framework_used
                false,  // came_from_commonjs (star re-exports are always ESM)
                SourceSpan::new(module_id.as_path(), 0, 0),  // No span available
            ));
        }
    }

    exports
}

fn convert_imports(normal: &NormalModule, module_id: &ModuleId) -> Vec<PendingImport> {
    let module_path = module_id.as_path().to_path_buf();
    let mut record_specifiers: FxHashMap<ImportRecordIdx, Vec<ImportSpecifier>> =
        FxHashMap::default();
    for named in normal.ecma_view.named_imports.values() {
        let entry = record_specifiers.entry(named.record_id).or_default();
        let specifier = match &named.imported {
            Specifier::Star => ImportSpecifier::Namespace("*".into()),
            Specifier::Literal(lit) => {
                if lit.as_str() == "default" {
                    ImportSpecifier::Default
                } else {
                    ImportSpecifier::Named(lit.to_string())
                }
            }
        };
        entry.push(specifier);
    }

    let mut span_lookup: FxHashMap<ImportRecordIdx, SourceSpan> = FxHashMap::default();
    for (span, record_idx) in &normal.ecma_view.imports {
        span_lookup.insert(
            *record_idx,
            SourceSpan::new(&module_path, span.start, span.end),
        );
    }

    // Build a set of exported names to detect re-exports
    let exported_names: std::collections::HashSet<String> = normal
        .ecma_view
        .named_exports
        .keys()
        .map(|name| name.to_string())
        .collect();
    
    normal
        .ecma_view
        .import_records
        .iter_enumerated()
        .map(|(idx, record)| {
            let specifiers = record_specifiers.remove(&idx).unwrap_or_default();
            
            // Check if this import is a re-export
            // Primary signal: Check if this import record has the IsExportStar flag
            // Secondary: Check if imported names match exported names (for named re-exports)
            let is_reexport = record.meta.contains(ImportRecordMeta::IsExportStar) || {
                // Named re-exports: check if imported names match exported names
                !specifiers.is_empty() && specifiers.iter().any(|spec| {
                    match spec {
                        ImportSpecifier::Named(name) => exported_names.contains(name),
                        ImportSpecifier::Default => exported_names.contains("default"),
                        ImportSpecifier::Namespace(_) => {
                            // Namespace imports (import * as X) are NOT re-exports
                            // unless they're also in the exported names
                            false
                        }
                    }
                })
            };
            
            let import_kind = if is_reexport {
                ImportKind::ReExport
            } else {
                convert_import_kind(record.kind)
            };
            
            let span = span_lookup
                .remove(&idx)
                .unwrap_or_else(|| SourceSpan::new(&module_path, 0, 0));
            let import = Import::new(
                record.module_request.to_string(),
                specifiers,
                import_kind,
                None,
                span,
            );
            PendingImport {
                import,
                target: Some(record.resolved_module),
            }
        })
        .collect()
}

fn convert_import_kind(kind: RdImportKind) -> ImportKind {
    match kind {
        RdImportKind::Import => ImportKind::Static,
        RdImportKind::DynamicImport => ImportKind::Dynamic,
        RdImportKind::Require => ImportKind::Require,
        RdImportKind::AtImport
        | RdImportKind::UrlImport
        | RdImportKind::NewUrl
        | RdImportKind::HotAccept => ImportKind::Static,
    }
}

fn convert_module_format(format: RdModuleDefFormat) -> FobModuleFormat {
    match format {
        RdModuleDefFormat::EsmMjs => FobModuleFormat::EsmMjs,
        RdModuleDefFormat::EsmPackageJson => FobModuleFormat::EsmPackageJson,
        RdModuleDefFormat::CjsPackageJson => FobModuleFormat::CjsPackageJson,
        RdModuleDefFormat::Unknown => FobModuleFormat::Unknown,
        // Rolldown uses variants like ESM and CJS (capitalized), map them to our format
        _ => FobModuleFormat::Unknown,  // Catch-all for any other variants
    }
}

fn convert_exports_kind(kind: RdExportsKind) -> FobExportsKind {
    match kind {
        RdExportsKind::Esm => FobExportsKind::Esm,
        RdExportsKind::CommonJs => FobExportsKind::CommonJs,
        RdExportsKind::None => FobExportsKind::None,
    }
}
