//! Symbol tracking for intra-file dead code analysis.
//!
//! This module provides types for tracking symbols (variables, functions, classes, etc.)
//! within individual modules, enabling detection of unused declarations and unreachable code.

mod metadata;
mod statistics;

use serde::{Deserialize, Serialize};

use super::ModuleId;

pub use metadata::{
    ClassMemberMetadata, CodeQualityMetadata, EnumMemberMetadata, EnumMemberValue, SymbolMetadata,
    Visibility,
};
pub use statistics::SymbolStatistics;

/// A symbol declared within a module (variable, function, class, etc.).
///
/// Symbols track their usage patterns (reads/writes) and scope information
/// to enable dead code detection within files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Symbol name (identifier)
    pub name: String,
    /// Kind of symbol (variable, function, class, etc.)
    pub kind: SymbolKind,
    /// Source location where the symbol is declared
    pub declaration_span: SymbolSpan,
    /// Number of times the symbol is read/referenced
    pub read_count: usize,
    /// Number of times the symbol is written/assigned
    pub write_count: usize,
    /// Whether this symbol is exported from the module
    pub is_exported: bool,
    /// Scope ID from Oxc semantic analysis
    pub scope_id: u32,
    /// Additional metadata for specialized symbol kinds
    #[serde(default)]
    pub metadata: SymbolMetadata,
}

impl Symbol {
    /// Create a new symbol with default usage counts and no metadata.
    pub fn new(
        name: String,
        kind: SymbolKind,
        declaration_span: SymbolSpan,
        scope_id: u32,
    ) -> Self {
        Self {
            name,
            kind,
            declaration_span,
            read_count: 0,
            write_count: 0,
            is_exported: false,
            scope_id,
            metadata: SymbolMetadata::None,
        }
    }

    /// Create a symbol with metadata
    pub fn with_metadata(
        name: String,
        kind: SymbolKind,
        declaration_span: SymbolSpan,
        scope_id: u32,
        metadata: SymbolMetadata,
    ) -> Self {
        Self {
            name,
            kind,
            declaration_span,
            read_count: 0,
            write_count: 0,
            is_exported: false,
            scope_id,
            metadata,
        }
    }

    /// Check if the symbol appears to be unused (no reads, only declarations).
    ///
    /// Exported symbols are never considered unused as they may be used externally.
    pub fn is_unused(&self) -> bool {
        !self.is_exported && self.read_count == 0 && self.write_count <= 1
    }

    /// Mark this symbol as exported.
    pub fn mark_exported(&mut self) {
        self.is_exported = true;
    }

    /// Check if this is a private class member that is unused
    ///
    /// This is the key insight for Danny: private members are safe to remove
    /// when unused, while public members might be used externally.
    pub fn is_unused_private_member(&self) -> bool {
        if !self.is_unused() {
            return false;
        }

        match &self.metadata {
            SymbolMetadata::ClassMember(meta) => matches!(meta.visibility, Visibility::Private),
            _ => false,
        }
    }

    /// Get the class this member belongs to (if it's a class member)
    pub fn class_name(&self) -> Option<&str> {
        match &self.metadata {
            SymbolMetadata::ClassMember(meta) => Some(&meta.class_name),
            _ => None,
        }
    }

    /// Check if this is a static class member
    pub fn is_static(&self) -> bool {
        match &self.metadata {
            SymbolMetadata::ClassMember(meta) => meta.is_static,
            _ => false,
        }
    }

    /// Check if this is an unused enum member
    pub fn is_unused_enum_member(&self) -> bool {
        matches!(self.kind, SymbolKind::EnumMember) && self.is_unused()
    }

    /// Get the enum this member belongs to (if it's an enum member)
    pub fn enum_name(&self) -> Option<&str> {
        match &self.metadata {
            SymbolMetadata::EnumMember(meta) => Some(&meta.enum_name),
            _ => None,
        }
    }

    /// Get code quality metadata if available
    pub fn code_quality_metadata(&self) -> Option<&CodeQualityMetadata> {
        match &self.metadata {
            SymbolMetadata::CodeQuality(meta) => Some(meta),
            _ => None,
        }
    }

    /// Get line count from code quality metadata
    pub fn line_count(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.line_count)
    }

    /// Get parameter count from code quality metadata (for functions)
    pub fn parameter_count(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.parameter_count)
    }

    /// Get cyclomatic complexity from code quality metadata (for functions)
    pub fn complexity(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.complexity)
    }

    /// Get maximum nesting depth from code quality metadata
    pub fn max_nesting_depth(&self) -> Option<usize> {
        self.code_quality_metadata()
            .and_then(|m| m.max_nesting_depth)
    }

    /// Get return count from code quality metadata (for functions)
    pub fn return_count(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.return_count)
    }

    /// Get method count from code quality metadata (for classes)
    pub fn method_count(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.method_count)
    }

    /// Get field count from code quality metadata (for classes)
    pub fn field_count(&self) -> Option<usize> {
        self.code_quality_metadata().and_then(|m| m.field_count)
    }
}

/// Classification of symbol types for better diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SymbolKind {
    /// Variable declaration (let, const, var)
    Variable,
    /// Function declaration or expression
    Function,
    /// Class declaration
    Class,
    /// Function parameter
    Parameter,
    /// TypeScript type alias
    TypeAlias,
    /// TypeScript interface
    Interface,
    /// Enum declaration
    Enum,
    /// Import binding
    Import,
    /// Class property (field)
    ClassProperty,
    /// Class method
    ClassMethod,
    /// Class getter
    ClassGetter,
    /// Class setter
    ClassSetter,
    /// Class constructor
    ClassConstructor,
    /// Enum member
    EnumMember,
}

impl SymbolKind {
    /// Returns true if this symbol kind can be safely removed when unused.
    ///
    /// Some symbols (like imports with side effects) should be handled carefully.
    pub fn is_safely_removable(&self) -> bool {
        matches!(
            self,
            Self::Variable | Self::Function | Self::Class | Self::TypeAlias | Self::Interface
        )
    }
}

/// Source location information for a symbol.
///
/// Simplified span tracking that doesn't require `oxc_span` types in the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolSpan {
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (0-indexed)
    pub column: u32,
    /// Byte offset in source
    pub offset: u32,
}

impl SymbolSpan {
    /// Create a new symbol span.
    pub fn new(line: u32, column: u32, offset: u32) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Create a zero-position span (for synthetic symbols).
    pub fn zero() -> Self {
        Self {
            line: 0,
            column: 0,
            offset: 0,
        }
    }
}

/// Collection of all symbols in a module.
///
/// This is the primary output of semantic analysis for a single file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SymbolTable {
    /// All declared symbols in the module
    pub symbols: Vec<Symbol>,
    /// Number of scopes in the module (from Oxc)
    pub scope_count: usize,
}

impl SymbolTable {
    /// Create a new empty symbol table.
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            scope_count: 0,
        }
    }

    /// Create a symbol table with a known capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            symbols: Vec::with_capacity(capacity),
            scope_count: 0,
        }
    }

    /// Add a symbol to the table.
    pub fn add_symbol(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    /// Get all unused symbols in this table.
    pub fn unused_symbols(&self) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.is_unused()).collect()
    }

    /// Get symbols by name.
    pub fn symbols_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.symbols.iter().filter(|s| s.name == name).collect()
    }

    /// Total number of symbols.
    pub fn len(&self) -> usize {
        self.symbols.len()
    }

    /// Check if the table is empty.
    pub fn is_empty(&self) -> bool {
        self.symbols.is_empty()
    }

    /// Mark symbols as exported if their names appear in the export list.
    pub fn mark_exports(&mut self, export_names: &[String]) {
        for symbol in &mut self.symbols {
            if export_names.contains(&symbol.name) {
                symbol.mark_exported();
            }
        }
    }

    /// Get all enum members grouped by enum name
    pub fn enum_members_by_enum(&self) -> std::collections::HashMap<String, Vec<&Symbol>> {
        use std::collections::HashMap;
        let mut result: HashMap<String, Vec<&Symbol>> = HashMap::new();

        for symbol in &self.symbols {
            if let SymbolMetadata::EnumMember(meta) = &symbol.metadata {
                result
                    .entry(meta.enum_name.clone())
                    .or_default()
                    .push(symbol);
            }
        }

        result
    }

    /// Get unused enum members
    pub fn unused_enum_members(&self) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.is_unused_enum_member())
            .collect()
    }
}

/// An unused symbol in a specific module.
///
/// This is returned by graph-level queries to provide context about where
/// the unused symbol is located.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedSymbol {
    /// The module containing the unused symbol
    pub module_id: ModuleId,
    /// The unused symbol itself
    pub symbol: Symbol,
}

/// Unreachable code detected in a module.
///
/// This represents code that can never be executed (e.g., after a return statement).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnreachableCode {
    /// The module containing unreachable code
    pub module_id: ModuleId,
    /// Description of the unreachable code
    pub description: String,
    /// Source location
    pub span: SymbolSpan,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_is_unused() {
        let mut symbol = Symbol::new(
            "unused_var".to_string(),
            SymbolKind::Variable,
            SymbolSpan::zero(),
            0,
        );

        // Declared but never read
        assert!(symbol.is_unused());

        // Read once
        symbol.read_count = 1;
        assert!(!symbol.is_unused());

        // Exported symbols are never unused
        let mut exported = Symbol::new(
            "exported_fn".to_string(),
            SymbolKind::Function,
            SymbolSpan::zero(),
            0,
        );
        exported.mark_exported();
        assert!(!exported.is_unused());
    }

    #[test]
    fn test_symbol_table_unused_symbols() {
        let mut table = SymbolTable::new();

        table.add_symbol(Symbol::new(
            "used".to_string(),
            SymbolKind::Variable,
            SymbolSpan::zero(),
            0,
        ));

        let used_symbol = table.symbols.last_mut().unwrap();
        used_symbol.read_count = 1;

        table.add_symbol(Symbol::new(
            "unused".to_string(),
            SymbolKind::Function,
            SymbolSpan::zero(),
            1,
        ));

        let unused = table.unused_symbols();
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].name, "unused");
    }

    #[test]
    fn test_mark_exports() {
        let mut table = SymbolTable::new();

        table.add_symbol(Symbol::new(
            "exported_fn".to_string(),
            SymbolKind::Function,
            SymbolSpan::zero(),
            0,
        ));

        table.add_symbol(Symbol::new(
            "internal".to_string(),
            SymbolKind::Variable,
            SymbolSpan::zero(),
            1,
        ));

        table.mark_exports(&["exported_fn".to_string()]);

        assert!(table.symbols[0].is_exported);
        assert!(!table.symbols[1].is_exported);
    }

    #[test]
    fn test_symbol_statistics() {
        let mut table1 = SymbolTable::new();
        table1.add_symbol(Symbol::new(
            "used".to_string(),
            SymbolKind::Function,
            SymbolSpan::zero(),
            0,
        ));
        table1.symbols[0].read_count = 1;

        let mut table2 = SymbolTable::new();
        table2.add_symbol(Symbol::new(
            "unused".to_string(),
            SymbolKind::Variable,
            SymbolSpan::zero(),
            0,
        ));

        let stats = SymbolStatistics::from_tables([&table1, &table2].iter().copied());
        assert_eq!(stats.total_symbols, 2);
        assert_eq!(stats.unused_symbols, 1);
        assert_eq!(stats.unused_percentage(), 50.0);
    }
}
