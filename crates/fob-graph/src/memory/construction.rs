//! Construction methods for ModuleGraph.

use rustc_hash::FxHashMap as HashMap;

use super::super::external_dep::ExternalDependency;
use super::super::{Module, ModuleId};
use super::graph::{GraphInner, ModuleGraph};
use fob::{Error, Result};

impl ModuleGraph {
    /// Create a new empty graph.
    pub fn new() -> Result<Self> {
        Ok(Self {
            inner: std::sync::Arc::new(parking_lot::RwLock::new(GraphInner::default())),
        })
    }

    /// Construct a graph from an iterator of modules (without edges).
    pub fn from_modules<I>(modules: I) -> Result<Self>
    where
        I: IntoIterator<Item = Module>,
    {
        let graph = Self::new()?;
        for module in modules {
            graph.add_module(module)?;
        }
        Ok(graph)
    }

    /// Create a ModuleGraph from collected module data.
    pub fn from_collected_data(
        collection: super::super::collection::CollectionState,
    ) -> Result<Self> {
        use super::super::from_collection::{
            convert_collected_exports, convert_collected_imports, convert_collected_module_id,
            has_star_export, infer_exports_kind, PendingImport,
        };
        use super::super::module::ModuleFormat as FobModuleFormat;
        use super::super::semantic::analyze_symbols;

        let graph = Self::new()?;

        let mut path_to_id: std::collections::HashMap<String, ModuleId> =
            std::collections::HashMap::new();
        let mut pending_modules: Vec<(ModuleId, Module, Vec<PendingImport>)> = Vec::new();

        // First pass: create module IDs and identify externals
        for (path, collected) in &collection.modules {
            if collected.is_external {
                continue;
            }

            let module_id = convert_collected_module_id(path)
                .map_err(|e| Error::InvalidConfig(format!("Module ID conversion failed: {}", e)))?;
            path_to_id.insert(path.clone(), module_id);
        }

        // Second pass: convert modules
        for (path, collected) in &collection.modules {
            if collected.is_external {
                continue;
            }

            let module_id = path_to_id.get(path).ok_or_else(|| {
                Error::InvalidConfig(format!("Module ID not found for path: {}", path))
            })?;

            let exports = convert_collected_exports(collected, module_id);
            let imports = convert_collected_imports(collected, module_id, &path_to_id);

            let has_side_effects = collected.has_side_effects;

            // Perform semantic analysis to extract symbols
            let source_type = super::super::SourceType::from_path(module_id.as_path());
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
            .module_format(FobModuleFormat::Unknown)
            .exports_kind(infer_exports_kind(&collected.exports))
            .has_star_exports(has_star_export(&collected.exports))
            .execution_order(None);

            if collected.is_entry {
                builder = builder.entry(true);
            }

            let module = builder.build();
            pending_modules.push((module_id.clone(), module, imports));
        }

        let mut external_aggregate: HashMap<String, ExternalDependency> = HashMap::default();

        // Third pass: resolve imports and build graph
        for (module_id, mut module, pending_imports) in pending_modules {
            let mut resolved_imports = Vec::with_capacity(pending_imports.len());
            for mut pending_import in pending_imports {
                if let Some(target_path) = pending_import.target {
                    if let Some(target_id) = path_to_id.get(&target_path) {
                        pending_import.import.resolved_to = Some(target_id.clone());
                        graph.add_dependency(module_id.clone(), target_id.clone())?;
                    } else {
                        // Any unresolved import is treated as external dependency
                        let dep = external_aggregate
                            .entry(target_path.clone())
                            .or_insert_with(|| ExternalDependency::new(target_path.clone()));
                        dep.push_importer(module_id.clone());
                    }
                }
                resolved_imports.push(pending_import.import);
            }

            // Update module with resolved imports before adding
            module.imports = std::sync::Arc::new(resolved_imports);
            graph.add_module(module)?;
        }

        for dep in external_aggregate.into_values() {
            graph.add_external_dependency(dep)?;
        }

        Ok(graph)
    }
}
