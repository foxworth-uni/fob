//! In-memory ModuleGraph implementation for WASM targets.
//!
//! This provides a HashMap-based graph storage that doesn't require SurrealDB,
//! making it compatible with WASM environments where native dependencies are limited.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::external_dep::ExternalDependency;
use super::import::{ImportKind, ImportSpecifier};
use super::symbol::{
    ClassMemberMetadata, EnumMemberValue, Symbol, SymbolMetadata,
    SymbolStatistics, UnreachableCode, UnusedSymbol,
};
use super::{Export, ExportKind, GraphStatistics, Import, Module, ModuleId, SourceSpan};
use crate::Result;

/// Information about a class member symbol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassMemberInfo {
    pub module_id: ModuleId,
    pub symbol: Symbol,
    pub metadata: ClassMemberMetadata,
}

/// Information about an enum member
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumMemberInfo {
    pub module_id: ModuleId,
    pub symbol: Symbol,
    pub value: Option<EnumMemberValue>,
}

/// A side-effect-only import (no bindings, just execution)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SideEffectImport {
    pub importer: ModuleId,
    pub source: String,
    pub resolved_to: Option<ModuleId>,
    pub span: SourceSpan,
}

/// Information about a namespace import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamespaceImportInfo {
    pub importer: ModuleId,
    pub namespace_name: String,
    pub source: String,
    pub resolved_to: Option<ModuleId>,
}

/// A type-only import (TypeScript)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeOnlyImport {
    pub importer: ModuleId,
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
    pub span: SourceSpan,
}

/// In-memory module dependency graph.
///
/// This implementation uses HashMaps for fast lookups and is fully synchronous.
/// All async methods are kept for API compatibility but don't actually await.
#[derive(Debug, Clone)]
pub struct ModuleGraph {
    inner: Arc<RwLock<GraphInner>>,
}

#[derive(Debug, Clone, Default)]
struct GraphInner {
    /// All modules indexed by ID
    modules: HashMap<ModuleId, Module>,
    /// Forward edges: module -> its dependencies
    dependencies: HashMap<ModuleId, Vec<ModuleId>>,
    /// Reverse edges: module -> modules that depend on it
    dependents: HashMap<ModuleId, Vec<ModuleId>>,
    /// Entry point modules
    entry_points: HashSet<ModuleId>,
    /// External dependencies
    external_deps: HashMap<String, ExternalDependency>,
}

impl ModuleGraph {
    /// Create a new empty graph.
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(GraphInner::default())),
        })
    }

    /// Create a graph with a specific storage path (no-op for memory implementation).
    pub async fn with_path(_path: Option<std::path::PathBuf>) -> Result<Self> {
        Self::new().await
    }

    /// Construct a graph from an iterator of modules (without edges).
    pub async fn from_modules<I>(modules: I) -> Result<Self>
    where
        I: IntoIterator<Item = Module>,
    {
        let graph = Self::new().await?;
        for module in modules {
            graph.add_module(module).await?;
        }
        Ok(graph)
    }

    /// Add a module into the graph.
    pub async fn add_module(&self, module: Module) -> Result<()> {
        let mut inner = self.inner.write();

        // Track if it's an entry point
        if module.is_entry {
            inner.entry_points.insert(module.id.clone());
        }

        // Track external dependencies (imports without resolved_to)
        for import in &module.imports {
            if import.resolved_to.is_none() && !import.source.is_empty() {
                // This is an unresolved import - treat as external dependency
                let dep = inner
                    .external_deps
                    .entry(import.source.clone())
                    .or_insert_with(|| ExternalDependency::new(import.source.clone()));
                dep.push_importer(module.id.clone());
            }
        }

        // Store the module
        inner.modules.insert(module.id.clone(), module);

        Ok(())
    }

    /// Add a dependency edge, creating forward and reverse mappings.
    pub async fn add_dependency(&self, from: ModuleId, to: ModuleId) -> Result<()> {
        let mut inner = self.inner.write();

        // Add forward edge
        inner.dependencies
            .entry(from.clone())
            .or_insert_with(Vec::new)
            .push(to.clone());

        // Add reverse edge
        inner.dependents
            .entry(to)
            .or_insert_with(Vec::new)
            .push(from);

        Ok(())
    }

    /// Add multiple dependencies from a single module.
    pub async fn add_dependencies<I>(&self, from: ModuleId, targets: I) -> Result<()>
    where
        I: IntoIterator<Item = ModuleId>,
    {
        for target in targets {
            self.add_dependency(from.clone(), target).await?;
        }
        Ok(())
    }

    /// Mark a module as an entry point.
    pub async fn add_entry_point(&self, id: ModuleId) -> Result<()> {
        let mut inner = self.inner.write();
        inner.entry_points.insert(id.clone());

        // Update the module itself if it exists
        if let Some(module) = inner.modules.get_mut(&id) {
            module.is_entry = true;
        }

        Ok(())
    }

    /// Add an external dependency record.
    pub async fn add_external_dependency(&self, dep: ExternalDependency) -> Result<()> {
        let mut inner = self.inner.write();
        inner.external_deps.insert(dep.specifier.clone(), dep);
        Ok(())
    }

    /// Fetch a module by ID.
    pub async fn module(&self, id: &ModuleId) -> Result<Option<Module>> {
        let inner = self.inner.read();
        Ok(inner.modules.get(id).cloned())
    }

    /// Get module by filesystem path.
    pub async fn module_by_path(&self, path: &Path) -> Result<Option<Module>> {
        let inner = self.inner.read();
        Ok(inner.modules.values().find(|module| module.path == path).cloned())
    }

    /// Get all modules.
    pub async fn modules(&self) -> Result<Vec<Module>> {
        let inner = self.inner.read();
        Ok(inner.modules.values().cloned().collect())
    }

    /// Dependencies of a module (forward edges).
    pub async fn dependencies(&self, id: &ModuleId) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner.dependencies.get(id).cloned().unwrap_or_default())
    }

    /// Dependents of a module (reverse edges).
    pub async fn dependents(&self, id: &ModuleId) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner.dependents.get(id).cloned().unwrap_or_default())
    }

    /// Modules importing the given module, returning a vector.
    pub async fn dependents_iter(&self, id: &ModuleId) -> Result<Vec<ModuleId>> {
        self.dependents(id).await
    }

    /// Whether a module is present.
    pub async fn contains(&self, id: &ModuleId) -> Result<bool> {
        let inner = self.inner.read();
        Ok(inner.modules.contains_key(id))
    }

    /// Entry points set.
    pub async fn entry_points(&self) -> Result<Vec<ModuleId>> {
        let inner = self.inner.read();
        Ok(inner.entry_points.iter().cloned().collect())
    }

    /// Return total module count.
    pub async fn len(&self) -> Result<usize> {
        let inner = self.inner.read();
        Ok(inner.modules.len())
    }

    /// Check whether graph is empty.
    pub async fn is_empty(&self) -> Result<bool> {
        let inner = self.inner.read();
        Ok(inner.modules.is_empty())
    }

    /// Get imports for a module.
    pub async fn imports_for_module(&self, id: &ModuleId) -> Result<Option<Vec<Import>>> {
        let inner = self.inner.read();
        Ok(inner.modules.get(id).map(|m| m.imports.clone()))
    }

    /// Aggregate external dependencies based on import data.
    pub async fn external_dependencies(&self) -> Result<Vec<ExternalDependency>> {
        let inner = self.inner.read();
        Ok(inner.external_deps.values().cloned().collect())
    }

    /// Compute modules with no dependents and no side effects.
    pub async fn unreachable_modules(&self) -> Result<Vec<Module>> {
        let inner = self.inner.read();
        let mut unreachable = Vec::new();

        for module in inner.modules.values() {
            if module.is_entry || module.has_side_effects {
                continue;
            }

            let has_dependents = inner.dependents
                .get(&module.id)
                .map(|deps| !deps.is_empty())
                .unwrap_or(false);

            if !has_dependents {
                unreachable.push(module.clone());
            }
        }

        Ok(unreachable)
    }

    /// Discover unused exports, respecting framework markers and namespace imports.
    pub async fn unused_exports(&self) -> Result<Vec<super::UnusedExport>> {
        let inner = self.inner.read();
        let mut unused = Vec::new();

        for module in inner.modules.values() {
            if module.is_entry {
                continue;
            }

            for export in &module.exports {
                if export.is_framework_used {
                    continue;
                }

                if !self.is_export_used_inner(&inner, &module.id, &export.name)? {
                    unused.push(super::UnusedExport {
                        module_id: module.id.clone(),
                        export: export.clone(),
                    });
                }
            }
        }

        Ok(unused)
    }

    fn is_export_used_inner(
        &self,
        inner: &GraphInner,
        module_id: &ModuleId,
        export_name: &str,
    ) -> Result<bool> {
        let dependents = inner.dependents.get(module_id).cloned().unwrap_or_default();

        for importer_id in dependents {
            if let Some(importer) = inner.modules.get(&importer_id) {
                for import_record in &importer.imports {
                    if import_record.resolved_to.as_ref() != Some(module_id) {
                        continue;
                    }

                    if import_record.specifiers.is_empty() {
                        // Side-effect import does not use exports.
                        continue;
                    }

                    let is_used = import_record.specifiers.iter().any(|specifier| match specifier {
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
            crate::Error::InvalidConfig(format!("Module {} not found in graph", module_id))
        })?;
        let source_path = source_module.path.to_string_lossy();

        for (re_exporter_id, re_exporter_module) in &inner.modules {
            for export in &re_exporter_module.exports {
                match export.kind {
                    ExportKind::StarReExport => {
                        // Star re-export: check if it's from our module
                        if let Some(ref re_exported_from) = export.re_exported_from {
                            if re_exported_from == source_path.as_ref() {
                                // This module re-exports all exports from our module
                                // Recursively check if this re-exporting module's export is used
                                if self.is_export_used_inner(inner, re_exporter_id, export_name)? {
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
                                    if self.is_export_used_inner(inner, re_exporter_id, &export.name)? {
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
    ///
    /// # Example
    /// ```ignore
    /// graph.compute_export_usage_counts().await?;
    ///
    /// for module in graph.modules().await? {
    ///     for export in &module.exports {
    ///         match export.usage_count() {
    ///             Some(0) => println!("Unused: {}", export.name),
    ///             Some(n) => println!("Used {} times: {}", n, export.name),
    ///             None => println!("Not computed: {}", export.name),
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn compute_export_usage_counts(&self) -> Result<()> {
        let mut inner = self.inner.write();

        // Collect module IDs first to avoid borrow checker issues
        let module_ids: Vec<ModuleId> = inner.modules.keys().cloned().collect();

        // For each module, compute usage counts for its exports
        for module_id in module_ids {
            // Get the module and its exports
            if let Some(module) = inner.modules.get(&module_id).cloned() {
                let mut updated_module = module.clone();

                // For each export, count how many times it's imported
                for export in &mut updated_module.exports {
                    let count = self.count_export_usage_inner(&inner, &module.id, &export.name)?;
                    export.set_usage_count(count);
                }

                // Update the module in the graph
                inner.modules.insert(module_id, updated_module);
            }
        }

        Ok(())
    }

    /// Helper method to count how many times a specific export is imported.
    ///
    /// This is the in-memory version that works with GraphInner.
    fn count_export_usage_inner(
        &self,
        inner: &GraphInner,
        module_id: &ModuleId,
        export_name: &str,
    ) -> Result<usize> {
        let dependents = inner.dependents.get(module_id).cloned().unwrap_or_default();
        let mut count = 0;

        for importer_id in dependents {
            if let Some(importer) = inner.modules.get(&importer_id) {
                for import_record in &importer.imports {
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

    /// Returns true if `from` depends on `to` (directly or transitively).
    pub async fn depends_on(&self, from: &ModuleId, to: &ModuleId) -> Result<bool> {
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
    pub async fn transitive_dependencies(&self, id: &ModuleId) -> Result<HashSet<ModuleId>> {
        let mut visited = HashSet::default();
        let mut queue = VecDeque::new();
        queue.push_back(id.clone());

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }

            let deps = self.dependencies(&current).await?;
            for next in deps {
                if !visited.contains(&next) {
                    queue.push_back(next);
                }
            }
        }

        visited.remove(id);
        Ok(visited)
    }

    /// Export the graph as DOT format for visualization.
    pub async fn to_dot_format(&self) -> Result<String> {
        let mut output = String::from("digraph ModuleGraph {\n");
        let all_modules = self.modules().await?;

        for module in &all_modules {
            output.push_str("    \"");
            output.push_str(&escape_label(&module.id.path_string()));
            output.push_str("\";\n");
        }

        for module in &all_modules {
            let deps = self.dependencies(&module.id).await?;
            for target in deps {
                output.push_str("    \"");
                output.push_str(&escape_label(&module.id.path_string()));
                output.push_str("\" -> \"");
                output.push_str(&escape_label(&target.path_string()));
                output.push_str("\";\n");
            }
        }

        output.push_str("}\n");
        Ok(output)
    }

    /// Export the graph and modules to JSON.
    pub async fn to_json(&self) -> Result<String> {
        let all_modules = self.modules().await?;
        let entry_points = self.entry_points().await?;
        let external_deps = self.external_dependencies().await?;

        #[derive(serde::Serialize)]
        struct GraphJson {
            modules: Vec<Module>,
            entry_points: Vec<ModuleId>,
            external_deps: Vec<ExternalDependency>,
        }

        let graph_json = GraphJson {
            modules: all_modules,
            entry_points,
            external_deps,
        };

        serde_json::to_string_pretty(&graph_json)
            .map_err(|e| crate::Error::InvalidConfig(format!("Failed to serialize graph: {e}")))
    }

    /// Compute statistics snapshot for dashboards.
    pub async fn statistics(&self) -> Result<GraphStatistics> {
        let unused = self.unused_exports().await?;
        let unreachable = self.unreachable_modules().await?;
        let all_modules = self.modules().await?;
        let side_effect_module_count = all_modules.iter().filter(|m| m.has_side_effects).count();
        let external_dependency_count = self.external_dependencies().await?.len();
        let entry_points = self.entry_points().await?;

        Ok(GraphStatistics::new(
            all_modules.len(),
            entry_points.len(),
            external_dependency_count,
            side_effect_module_count,
            unused.len(),
            unreachable.len(),
        ))
    }

    /// Apply a custom framework rule.
    ///
    /// Framework rules mark exports as framework-used based on naming conventions.
    /// This prevents false-positive "unused export" warnings.
    pub async fn apply_framework_rule(&self, rule: Box<dyn super::FrameworkRule>) -> Result<()> {
        rule.apply(self).await
    }

    /// Apply multiple framework rules.
    pub async fn apply_framework_rules(&self, rules: Vec<Box<dyn super::FrameworkRule>>) -> Result<()> {
        for rule in rules {
            self.apply_framework_rule(rule).await?;
        }
        Ok(())
    }

    /// Check if a direct dependency exists between two modules.
    pub async fn has_dependency(&self, from: &ModuleId, to: &ModuleId) -> Result<bool> {
        let deps = self.dependencies(from).await?;
        Ok(deps.contains(to))
    }

    /// Get all framework-used exports in the graph.
    pub async fn framework_used_exports(&self) -> Result<Vec<(ModuleId, Export)>> {
        let mut result = Vec::new();
        let all_modules = self.modules().await?;

        for module in all_modules {
            for export in module.exports {
                if export.is_framework_used {
                    result.push((module.id.clone(), export));
                }
            }
        }

        Ok(result)
    }

    /// Get all unused symbols across the entire graph.
    ///
    /// This queries the symbol table for each module and returns symbols
    /// that are declared but never referenced.
    pub async fn unused_symbols(&self) -> Result<Vec<UnusedSymbol>> {
        let inner = self.inner.read();
        let mut unused = Vec::new();

        for module in inner.modules.values() {
            for symbol in module.symbol_table.unused_symbols() {
                unused.push(UnusedSymbol {
                    module_id: module.id.clone(),
                    symbol: symbol.clone(),
                });
            }
        }

        Ok(unused)
    }

    /// Get unused symbols for a specific module.
    ///
    /// Returns an empty vector if the module doesn't exist.
    pub async fn unused_symbols_in_module(&self, id: &ModuleId) -> Result<Vec<Symbol>> {
        let inner = self.inner.read();

        if let Some(module) = inner.modules.get(id) {
            Ok(module
                .symbol_table
                .unused_symbols()
                .into_iter()
                .cloned()
                .collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get all symbols across the entire graph (not just unused ones).
    ///
    /// This is useful for code quality analysis that needs to check all symbols,
    /// regardless of whether they're used or not.
    pub async fn all_symbols(&self) -> Result<Vec<UnusedSymbol>> {
        let inner = self.inner.read();
        let mut all = Vec::new();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                all.push(UnusedSymbol {
                    module_id: module.id.clone(),
                    symbol: symbol.clone(),
                });
            }
        }

        Ok(all)
    }

    /// Get unreachable code detected across the graph.
    ///
    /// Note: Unreachable code detection must be performed during module analysis
    /// (when source code is available) rather than from the graph.
    /// Use `crate::graph::semantic::detect_unreachable_code()` during module building.
    ///
    /// This method currently returns an empty vector as a placeholder for graph-level
    /// aggregation if unreachable code data is stored in modules in the future.
    pub async fn unreachable_code(&self) -> Result<Vec<UnreachableCode>> {
        // TODO: Aggregate unreachable code from modules if stored in Module struct
        // Currently modules don't store source text or unreachable code data
        Ok(Vec::new())
    }

    /// Compute symbol statistics across the entire graph.
    ///
    /// Aggregates symbol information from all modules to provide
    /// a high-level view of symbol usage patterns.
    pub async fn symbol_statistics(&self) -> Result<SymbolStatistics> {
        let inner = self.inner.read();

        let tables: Vec<_> = inner
            .modules
            .values()
            .map(|m| &m.symbol_table)
            .collect();

        Ok(SymbolStatistics::from_tables(tables.into_iter()))
    }

    /// Get all unused private class members across the graph, grouped by class.
    ///
    /// Private class members are safe to remove when unused, as they cannot be accessed
    /// from outside the class. This method groups results by class name for easier analysis.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::graph::ModuleGraph;
    /// # async fn example(graph: &ModuleGraph) -> fob_core::Result<()> {
    /// let unused_by_class = graph.unused_private_class_members().await?;
    /// for (class_name, members) in unused_by_class {
    ///     println!("Class {}: {} unused private members", class_name, members.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unused_private_class_members(&self) -> Result<HashMap<String, Vec<UnusedSymbol>>> {
        let inner = self.inner.read();
        let mut by_class: HashMap<String, Vec<UnusedSymbol>> = HashMap::default();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if symbol.is_unused_private_member() {
                    if let Some(class_name) = symbol.class_name() {
                        by_class
                            .entry(class_name.to_string())
                            .or_default()
                            .push(UnusedSymbol {
                                module_id: module.id.clone(),
                                symbol: symbol.clone(),
                            });
                    }
                }
            }
        }

        Ok(by_class)
    }

    /// Get all class members (public and private) for comprehensive analysis.
    ///
    /// This provides complete visibility into class structure, useful for refactoring
    /// and understanding class complexity.
    pub async fn all_class_members(&self) -> Result<Vec<ClassMemberInfo>> {
        let inner = self.inner.read();
        let mut members = Vec::new();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if let SymbolMetadata::ClassMember(metadata) = &symbol.metadata {
                    members.push(ClassMemberInfo {
                        module_id: module.id.clone(),
                        symbol: symbol.clone(),
                        metadata: metadata.clone(),
                    });
                }
            }
        }

        Ok(members)
    }

    /// Get unused public class members.
    ///
    /// Warning: Removing public members is potentially breaking for library code.
    /// Use with caution and only for application code where you control all usage.
    pub async fn unused_public_class_members(&self) -> Result<Vec<UnusedSymbol>> {
        let inner = self.inner.read();
        let mut unused = Vec::new();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if symbol.is_unused() {
                    if let SymbolMetadata::ClassMember(metadata) = &symbol.metadata {
                        if !matches!(metadata.visibility, super::symbol::Visibility::Private) {
                            unused.push(UnusedSymbol {
                                module_id: module.id.clone(),
                                symbol: symbol.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(unused)
    }

    /// Get all unused enum members across the graph, grouped by enum.
    ///
    /// Enum members that are never referenced can often be safely removed,
    /// especially in application code. This groups results by enum for easier analysis.
    pub async fn unused_enum_members(&self) -> Result<HashMap<String, Vec<UnusedSymbol>>> {
        let inner = self.inner.read();
        let mut by_enum: HashMap<String, Vec<UnusedSymbol>> = HashMap::default();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if symbol.is_unused_enum_member() {
                    if let Some(enum_name) = symbol.enum_name() {
                        by_enum
                            .entry(enum_name.to_string())
                            .or_default()
                            .push(UnusedSymbol {
                                module_id: module.id.clone(),
                                symbol: symbol.clone(),
                            });
                    }
                }
            }
        }

        Ok(by_enum)
    }

    /// Get all enum members (used and unused) for comprehensive enum analysis.
    ///
    /// This provides complete visibility into enum structure, useful for
    /// understanding enum coverage and usage patterns.
    pub async fn all_enum_members(&self) -> Result<HashMap<String, Vec<EnumMemberInfo>>> {
        let inner = self.inner.read();
        let mut by_enum: HashMap<String, Vec<EnumMemberInfo>> = HashMap::default();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if let SymbolMetadata::EnumMember(metadata) = &symbol.metadata {
                    by_enum
                        .entry(metadata.enum_name.clone())
                        .or_default()
                        .push(EnumMemberInfo {
                            module_id: module.id.clone(),
                            symbol: symbol.clone(),
                            value: metadata.value.clone(),
                        });
                }
            }
        }

        Ok(by_enum)
    }

    /// Detect unused npm dependencies by cross-referencing package.json with imports.
    ///
    /// This identifies dependencies declared in package.json but never actually imported
    /// in the codebase. Useful for cleaning up unused packages.
    ///
    /// # Parameters
    ///
    /// - `package_json`: Parsed package.json file
    /// - `include_dev`: Whether to check devDependencies
    /// - `include_peer`: Whether to check peerDependencies
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::graph::{ModuleGraph, PackageJson};
    /// # async fn example(graph: &ModuleGraph, pkg: &PackageJson) -> fob_core::Result<()> {
    /// let unused = graph.unused_npm_dependencies(pkg, true, false).await?;
    /// for dep in unused {
    ///     println!("Unused: {} ({})", dep.package, dep.dep_type.as_str());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unused_npm_dependencies(
        &self,
        package_json: &super::package_json::PackageJson,
        include_dev: bool,
        include_peer: bool,
    ) -> Result<Vec<super::package_json::UnusedDependency>> {
        use super::package_json::{DependencyType, UnusedDependency, extract_package_name};

        let inner = self.inner.read();

        // Collect all imported package names
        let mut imported_packages = HashSet::default();
        for module in inner.modules.values() {
            for import in &module.imports {
                if import.is_external() {
                    let package_name = extract_package_name(&import.source);
                    imported_packages.insert(package_name.to_string());
                }
            }
        }

        let mut unused = Vec::new();

        // Check each dependency type
        let dep_types = [
            (DependencyType::Production, true),
            (DependencyType::Development, include_dev),
            (DependencyType::Peer, include_peer),
            (DependencyType::Optional, true),
        ];

        for (dep_type, should_check) in dep_types {
            if !should_check {
                continue;
            }

            for (package, version) in package_json.get_dependencies(dep_type) {
                if !imported_packages.contains(package) {
                    unused.push(UnusedDependency {
                        package: package.clone(),
                        version: version.clone(),
                        dep_type,
                    });
                }
            }
        }

        Ok(unused)
    }

    /// Get dependency coverage statistics.
    ///
    /// Provides detailed metrics about which dependencies are actually used
    /// vs declared in package.json.
    pub async fn dependency_coverage(
        &self,
        package_json: &super::package_json::PackageJson,
    ) -> Result<super::package_json::DependencyCoverage> {
        use super::package_json::{DependencyCoverage, DependencyType, TypeCoverage, extract_package_name};

        let inner = self.inner.read();

        // Collect all imported package names
        let mut imported_packages = HashSet::default();
        for module in inner.modules.values() {
            for import in &module.imports {
                if import.is_external() {
                    let package_name = extract_package_name(&import.source);
                    imported_packages.insert(package_name.to_string());
                }
            }
        }

        let mut by_type = std::collections::HashMap::new();
        let mut total_declared = 0;
        let mut total_used = 0;

        for dep_type in [
            DependencyType::Production,
            DependencyType::Development,
            DependencyType::Peer,
            DependencyType::Optional,
        ] {
            let deps = package_json.get_dependencies(dep_type);
            let declared = deps.len();
            let used = deps
                .keys()
                .filter(|pkg| imported_packages.contains(*pkg))
                .count();
            let unused = declared - used;

            total_declared += declared;
            total_used += used;

            by_type.insert(
                dep_type,
                TypeCoverage {
                    declared,
                    used,
                    unused,
                },
            );
        }

        Ok(DependencyCoverage {
            total_declared,
            total_used,
            total_unused: total_declared - total_used,
            by_type,
        })
    }

    /// Find all dependency chains from entry points to a target module.
    ///
    /// This traces all possible paths through the import graph, useful for understanding
    /// why a module is included in the bundle and what depends on it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use fob_core::graph::{ModuleGraph, ModuleId};
    /// # async fn example(graph: &ModuleGraph, target: &ModuleId) -> fob_core::Result<()> {
    /// let chains = graph.dependency_chains_to(target).await?;
    /// for chain in chains {
    ///     println!("Path: {}", chain.format_chain());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dependency_chains_to(
        &self,
        target: &ModuleId,
    ) -> Result<Vec<super::dependency_chain::DependencyChain>> {
        use super::dependency_chain::find_chains;

        let entry_points = self.entry_points().await?;
        let inner = self.inner.read();

        let get_deps = |module: &ModuleId| -> Vec<ModuleId> {
            inner
                .dependencies
                .get(module)
                .cloned()
                .unwrap_or_default()
        };

        Ok(find_chains(&entry_points, target, get_deps))
    }

    /// Analyze dependency chains to a module.
    ///
    /// Provides comprehensive statistics about all paths leading to a module.
    pub async fn analyze_dependency_chains(
        &self,
        target: &ModuleId,
    ) -> Result<super::dependency_chain::ChainAnalysis> {
        use super::dependency_chain::ChainAnalysis;

        let chains = self.dependency_chains_to(target).await?;
        Ok(ChainAnalysis::from_chains(target.clone(), chains))
    }

    /// Get the import depth of a module from entry points.
    ///
    /// Returns the shortest distance from any entry point to this module,
    /// or None if the module is unreachable.
    pub async fn import_depth(&self, module: &ModuleId) -> Result<Option<usize>> {
        let analysis = self.analyze_dependency_chains(module).await?;
        Ok(analysis.min_depth)
    }

    /// Group modules by their import depth from entry points.
    ///
    /// This creates layers of the dependency graph, useful for visualizing
    /// the structure and understanding module organization.
    pub async fn modules_by_depth(&self) -> Result<HashMap<usize, Vec<ModuleId>>> {
        let all_modules = self.modules().await?;
        let mut by_depth: HashMap<usize, Vec<ModuleId>> = HashMap::default();

        for module in all_modules {
            if let Some(depth) = self.import_depth(&module.id).await? {
                by_depth.entry(depth).or_default().push(module.id);
            }
        }

        Ok(by_depth)
    }

    /// Check if a module is only reachable through dead code.
    ///
    /// A module is considered "reachable only through dead code" if:
    /// - It has no direct path from any entry point, OR
    /// - All paths to it go through unreachable modules
    ///
    /// This is a strong indicator that the module can be safely removed.
    pub async fn is_reachable_only_through_dead_code(&self, module: &ModuleId) -> Result<bool> {
        let analysis = self.analyze_dependency_chains(module).await?;

        // If not reachable at all, it's definitely dead
        if !analysis.is_reachable() {
            return Ok(true);
        }

        // If we have any chain, the module is reachable from an entry point
        // More sophisticated analysis would check if all chains go through
        // modules that are themselves unreachable, but that requires
        // recursive analysis which we'll skip for now.
        Ok(false)
    }

    /// Get all side-effect-only imports across the graph.
    ///
    /// Side-effect imports like `import 'polyfill'` execute code but don't bind any values.
    /// These are important to track as they can't be tree-shaken and always contribute
    /// to bundle size.
    pub async fn side_effect_only_imports(&self) -> Result<Vec<SideEffectImport>> {
        let inner = self.inner.read();
        let mut side_effects = Vec::new();

        for module in inner.modules.values() {
            for import in &module.imports {
                if import.is_side_effect_only() {
                    side_effects.push(SideEffectImport {
                        importer: module.id.clone(),
                        source: import.source.clone(),
                        resolved_to: import.resolved_to.clone(),
                        span: import.span.clone(),
                    });
                }
            }
        }

        Ok(side_effects)
    }

    /// Get all namespace imports and their usage.
    ///
    /// Namespace imports (`import * as foo`) import all exports from a module.
    /// This is useful for tracking potential over-imports that could be optimized
    /// to named imports.
    pub async fn namespace_imports(&self) -> Result<Vec<NamespaceImportInfo>> {
        let inner = self.inner.read();
        let mut namespaces = Vec::new();

        for module in inner.modules.values() {
            for import in &module.imports {
                if import.is_namespace_import() {
                    if let Some(namespace_name) = import.namespace_name() {
                        namespaces.push(NamespaceImportInfo {
                            importer: module.id.clone(),
                            namespace_name: namespace_name.to_string(),
                            source: import.source.clone(),
                            resolved_to: import.resolved_to.clone(),
                        });
                    }
                }
            }
        }

        Ok(namespaces)
    }

    /// Get all type-only imports (TypeScript).
    ///
    /// Type-only imports are erased at runtime and don't contribute to bundle size.
    /// Tracking these helps understand the TypeScript structure without conflating
    /// with runtime dependencies.
    pub async fn type_only_imports(&self) -> Result<Vec<TypeOnlyImport>> {
        let inner = self.inner.read();
        let mut type_imports = Vec::new();

        for module in inner.modules.values() {
            for import in &module.imports {
                if import.is_type_only() {
                    type_imports.push(TypeOnlyImport {
                        importer: module.id.clone(),
                        source: import.source.clone(),
                        specifiers: import.specifiers.clone(),
                        span: import.span.clone(),
                    });
                }
            }
        }

        Ok(type_imports)
    }
}

/// Helper to escape labels for DOT format.
fn escape_label(label: &str) -> String {
    label.replace('"', "\\\"")
}
