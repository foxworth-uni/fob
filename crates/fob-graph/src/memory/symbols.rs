//! Symbol analysis methods for ModuleGraph.

use rustc_hash::FxHashMap as HashMap;

use super::super::ModuleId;
use super::super::symbol::{
    Symbol, SymbolMetadata, SymbolStatistics, UnreachableCode, UnusedSymbol,
};
use super::graph::ModuleGraph;
use super::types::{ClassMemberInfo, EnumMemberInfo};
use fob_core::Result;

impl ModuleGraph {
    /// Get all unused symbols across the entire graph.
    ///
    /// This queries the symbol table for each module and returns symbols
    /// that are declared but never referenced.
    pub fn unused_symbols(&self) -> Result<Vec<UnusedSymbol>> {
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
    pub fn unused_symbols_in_module(&self, id: &ModuleId) -> Result<Vec<Symbol>> {
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
    pub fn all_symbols(&self) -> Result<Vec<UnusedSymbol>> {
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
    pub fn unreachable_code(&self) -> Result<Vec<UnreachableCode>> {
        // Architectural limitation: Unreachable code aggregation is deferred because:
        // 1. The Module struct doesn't store source text (by design for memory efficiency)
        // 2. Unreachable code detection requires source text analysis
        // 3. Storing unreachable code data in Module would increase memory usage
        //
        // Current approach: Use `crate::semantic::detect_unreachable_code()` during module
        // building when source text is available, rather than aggregating from the graph.
        //
        // Future enhancement: If graph-level aggregation is needed, consider:
        // - Adding optional unreachable_code field to Module (with source text caching)
        // - Or maintaining a separate map of module_id -> unreachable_code
        // - Or requiring callers to provide source text when querying
        Ok(Vec::new())
    }

    /// Compute symbol statistics across the entire graph.
    ///
    /// Aggregates symbol information from all modules to provide
    /// a high-level view of symbol usage patterns.
    pub fn symbol_statistics(&self) -> Result<SymbolStatistics> {
        let inner = self.inner.read();

        let tables: Vec<_> = inner
            .modules
            .values()
            .map(|m| m.symbol_table.as_ref())
            .collect();

        Ok(SymbolStatistics::from_tables(tables.into_iter()))
    }

    /// Get all unused private class members across the graph, grouped by class.
    ///
    /// Private class members are safe to remove when unused, as they cannot be accessed
    /// from outside the class. This method groups results by class name for easier analysis.
    pub fn unused_private_class_members(&self) -> Result<HashMap<String, Vec<UnusedSymbol>>> {
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
    pub fn all_class_members(&self) -> Result<Vec<ClassMemberInfo>> {
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
    pub fn unused_public_class_members(&self) -> Result<Vec<UnusedSymbol>> {
        let inner = self.inner.read();
        let mut unused = Vec::new();

        for module in inner.modules.values() {
            for symbol in &module.symbol_table.symbols {
                if symbol.is_unused() {
                    if let SymbolMetadata::ClassMember(metadata) = &symbol.metadata {
                        if !matches!(
                            metadata.visibility,
                            super::super::symbol::Visibility::Private
                        ) {
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
    pub fn unused_enum_members(&self) -> Result<HashMap<String, Vec<UnusedSymbol>>> {
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
    pub fn all_enum_members(&self) -> Result<HashMap<String, Vec<EnumMemberInfo>>> {
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
}
