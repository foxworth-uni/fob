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

/// Fast line/column lookup using pre-calculated line offsets.
pub(super) struct LineIndex {
    line_starts: Vec<u32>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (i, ch) in source.char_indices() {
            if ch == '\n' {
                line_starts.push((i + 1) as u32);
            }
        }
        Self { line_starts }
    }

    /// Calculate line and column from byte offset.
    /// Returns (line, column) where line is 1-indexed and column is 0-indexed.
    ///
    /// # Safety Note
    ///
    /// This function previously had an underflow bug when `offset=0`:
    /// `binary_search` would return `Err(0)`, and `0 - 1` would underflow.
    /// The fix explicitly handles `Err(0)` to map it to line index 0.
    pub fn get_line_column(&self, offset: u32, source: &str) -> (u32, u32) {
        // Binary search for the line
        // When offset is less than all line starts, binary_search returns Err(0).
        // We must handle this explicitly to prevent underflow from idx - 1.
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(0) => 0, // Offset before first line start (prevents underflow)
            Err(idx) => idx - 1,
        };

        let line_start = self.line_starts[line_idx] as usize;
        let line = (line_idx + 1) as u32;

        // Calculate column by counting characters from line start
        // This is safe because we know offset >= line_start
        let column = if offset as usize > source.len() {
            0
        } else {
            // We need character count, not byte count for column
            source[line_start..offset as usize].chars().count() as u32
        };

        (line, column)
    }
}
