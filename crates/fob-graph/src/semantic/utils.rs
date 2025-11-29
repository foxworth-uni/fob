//! Utility functions for semantic analysis.

use super::super::SourceType;
use super::super::symbol::SymbolKind;
use crate::oxc::SourceType as OxcSourceType;

/// Convert Fob's SourceType to Oxc's SourceType.
pub(super) fn convert_source_type(source_type: SourceType, filename: &str) -> OxcSourceType {
    match source_type {
        SourceType::JavaScript => {
            OxcSourceType::from_path(filename).unwrap_or(OxcSourceType::mjs())
        }
        SourceType::TypeScript => OxcSourceType::ts(),
        SourceType::Jsx => OxcSourceType::jsx(),
        SourceType::Tsx => OxcSourceType::tsx(),
        _ => OxcSourceType::mjs(),
    }
}

/// Determine the symbol kind from Oxc symbol flags.
pub(super) fn determine_symbol_kind(flags: oxc_semantic::SymbolFlags) -> SymbolKind {
    use oxc_semantic::SymbolFlags;

    // Check flags in order of specificity
    if flags.contains(SymbolFlags::Function) {
        SymbolKind::Function
    } else if flags.contains(SymbolFlags::Class) {
        SymbolKind::Class
    } else if flags.contains(SymbolFlags::TypeAlias) {
        SymbolKind::TypeAlias
    } else if flags.contains(SymbolFlags::Interface) {
        SymbolKind::Interface
    } else if flags.contains(SymbolFlags::RegularEnum) || flags.contains(SymbolFlags::ConstEnum) {
        SymbolKind::Enum
    } else if flags.contains(SymbolFlags::Import) {
        SymbolKind::Import
    } else if flags.contains(SymbolFlags::FunctionScopedVariable)
        || flags.contains(SymbolFlags::BlockScopedVariable)
        || flags.contains(SymbolFlags::ConstVariable)
    {
        SymbolKind::Variable
    } else {
        // Default to variable for unknown types
        SymbolKind::Variable
    }
}

/// Calculate line and column from byte offset in source text.
///
/// Returns (line, column) where line is 1-indexed and column is 0-indexed.
pub(super) fn get_line_column(source: &str, offset: u32) -> (u32, u32) {
    let offset = offset as usize;
    if offset > source.len() {
        return (0, 0);
    }

    let mut line = 1u32;
    let mut column = 0u32;
    let mut current_offset = 0;

    for ch in source.chars() {
        if current_offset >= offset {
            break;
        }

        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }

        current_offset += ch.len_utf8();
    }

    (line, column)
}
