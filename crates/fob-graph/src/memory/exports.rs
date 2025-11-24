//! Export analysis methods for ModuleGraph.

use std::sync::Arc;

use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::super::import::{ImportKind, ImportSpecifier};
use super::super::{ExportKind, Module, ModuleId};
use super::graph::{GraphInner, ModuleGraph};
use fob::{Error, Result};

impl ModuleGraph {
    /// Discover unused exports, respecting framework markers and namespace imports.
    pub fn unused_exports(&self) -> Result<Vec<super::super::UnusedExport>> {
        let inner = self.inner.read();
        let mut unused = Vec::new();

        for module in inner.modules.values() {
            if module.is_entry {
                continue;
            }

            for export in module.exports.iter() {
                if export.is_framework_used {
                    continue;
                }

                if !Self::is_export_used_inner(&inner, &module.id, &export.name)? {
                    unused.push(super::super::UnusedExport {
                        module_id: module.id.clone(),
                        export: export.clone(),
                    });
                }
            }
        }

        Ok(unused)
    }

    pub(super) fn is_export_used_inner(
        inner: &GraphInner,
        module_id: &ModuleId,
        export_name: &str,
    ) -> Result<bool> {
        let dependents = inner.dependents.get(module_id).cloned().unwrap_or_default();

        for importer_id in dependents {
            if let Some(importer) = inner.modules.get(&importer_id) {
                for import_record in importer.imports.iter() {
                    if import_record.resolved_to.as_ref() != Some(module_id) {
                        continue;
                    }

                    if import_record.specifiers.is_empty() {
                        // Side-effect import does not use exports.
                        continue;
                    }

                    let is_used =
                        import_record
                            .specifiers
                            .iter()
                            .any(|specifier| match specifier {
                                ImportSpecifier::Named(name) => name == export_name,
                                ImportSpecifier::Default => export_name == "default",
                                ImportSpecifier::Namespace(_) => {
                                    // True namespace imports (import * as X) use ALL exports
                                    // But star re-exports (export * from) only forward, not use
                                    !matches!(import_record.kind, ImportKind::ReExport)
                                }
                            });

                    if is_used {
                        return Ok(true);
                    }
                }
            }
        }

        // Check if this export is re-exported by other modules and used transitively
        // This handles cases like: validators.ts exports validateEmail -> helpers.ts does
        // export * from validators.ts -> demo.tsx imports { validateEmail } from helpers.ts

        // Get the source module's path for comparison (re_exported_from uses path, not module ID)
        let source_module = inner.modules.get(module_id).ok_or_else(|| {
            Error::InvalidConfig(format!("Module {} not found in graph", module_id))
        })?;
        let source_path = source_module.path.to_string_lossy();

        for (re_exporter_id, re_exporter_module) in &inner.modules {
            for export in re_exporter_module.exports.iter() {
                match export.kind {
                    ExportKind::StarReExport => {
                        // Star re-export: check if it's from our module
                        if let Some(ref re_exported_from) = export.re_exported_from {
                            if re_exported_from == source_path.as_ref() {
                                // This module re-exports all exports from our module
                                // Recursively check if this re-exporting module's export is used
                                if Self::is_export_used_inner(inner, re_exporter_id, export_name)? {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                    ExportKind::ReExport => {
                        // Named re-export: check if it matches our export
                        if export.name == export_name {
                            if let Some(ref re_exported_from) = export.re_exported_from {
                                if re_exported_from == source_path.as_ref() {
                                    // This is a named re-export of our specific export
                                    // Recursively check if THIS re-export is used
                                    if Self::is_export_used_inner(
                                        inner,
                                        re_exporter_id,
                                        &export.name,
                                    )? {
                                        return Ok(true);
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        // Named, Default, TypeOnly - not re-exports, skip
                    }
                }
            }
        }

        Ok(false)
    }

    /// Computes and sets usage counts for all exports in the module graph.
    ///
    /// For each export in each module, this counts how many times it's imported
    /// across all dependent modules and updates the `usage_count` field.
    ///
    /// Usage counts are determined by:
    /// - Named imports: Each `import { foo }` increments the count for export "foo"
    /// - Default imports: Each `import foo` increments the count for export "default"
    /// - Namespace imports: Each `import * as ns` increments the count for ALL exports by 1
    ///   (except star re-exports which only forward, not consume)
    /// - Re-exports: Counted separately as they create new import paths
    ///
    /// After calling this method, each Export will have `usage_count` set to:
    /// - `Some(0)` if the export is unused
    /// - `Some(n)` where n > 0 for the number of import sites
    pub fn compute_export_usage_counts(&self) -> Result<()> {
        // 1. Snapshot only IDs (not full HashMap)
        let module_ids: Vec<ModuleId> = {
            let inner = self.inner.read();
            inner.modules.keys().cloned().collect()
        };

        // 2. Process each module with brief read locks
        let mut updates = HashMap::default();
        for module_id in module_ids {
            let (module, dependents) = {
                let inner = self.inner.read();
                // Skip modules that were removed concurrently
                let Some(module_arc) = inner.modules.get(&module_id) else {
                    continue;
                };
                let module = (**module_arc).clone();
                let dependents = inner
                    .dependents
                    .get(&module_id)
                    .cloned()
                    .unwrap_or_default();
                (module, dependents)
            }; // Lock released here

            let mut updated_module = module;
            // Use Arc::make_mut to get mutable access to exports
            let exports = std::sync::Arc::make_mut(&mut updated_module.exports);
            for export in exports.iter_mut() {
                let count = {
                    // Get importer modules for counting
                    let inner = self.inner.read();
                    Self::count_export_usage_standalone(
                        &inner.modules,
                        &module_id,
                        &export.name,
                        &dependents,
                    )?
                };
                export.set_usage_count(count);
            }

            updates.insert(module_id, std::sync::Arc::new(updated_module));
        }

        // 3. Apply updates with single write lock
        {
            let mut inner = self.inner.write();
            for (id, module) in updates {
                inner.modules.insert(id, module);
            }
        }

        Ok(())
    }

    /// Standalone helper to count export usage.
    ///
    /// This works with a read lock on modules HashMap, counting how many times
    /// an export is imported by dependent modules.
    fn count_export_usage_standalone(
        modules: &HashMap<ModuleId, Arc<Module>>,
        module_id: &ModuleId,
        export_name: &str,
        dependents: &HashSet<ModuleId>,
    ) -> Result<usize> {
        let mut count = 0;

        for importer_id in dependents {
            if let Some(importer) = modules.get(importer_id) {
                for import_record in importer.imports.iter() {
                    if import_record.resolved_to.as_ref() != Some(module_id) {
                        continue;
                    }

                    if import_record.specifiers.is_empty() {
                        // Side-effect import does not use exports.
                        continue;
                    }

                    // Count matching specifiers
                    for specifier in &import_record.specifiers {
                        let matches = match specifier {
                            ImportSpecifier::Named(name) => name == export_name,
                            ImportSpecifier::Default => export_name == "default",
                            ImportSpecifier::Namespace(_) => {
                                // Namespace imports (import * as X) use ALL exports once
                                // But star re-exports (export * from) only forward, not use
                                !matches!(import_record.kind, ImportKind::ReExport)
                            }
                        };

                        if matches {
                            count += 1;
                            // For namespace imports, we only count once per import statement
                            // not once per export, so we break here
                            if matches!(specifier, ImportSpecifier::Namespace(_)) {
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(count)
    }
}
