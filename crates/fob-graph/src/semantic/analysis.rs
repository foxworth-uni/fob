//! Symbol analysis functions for extracting symbol information from source code.

use super::super::symbol::{Symbol, SymbolSpan, SymbolTable};
use super::utils::{determine_symbol_kind, get_line_column};
use fob_core::oxc::SemanticBuilder;

/// Extract symbols from parsed program using Oxc's semantic analysis.
///
/// This function performs the core symbol extraction logic:
/// - Builds semantic information from the parsed AST
/// - Extracts symbols from Oxc's scoping information
/// - Counts read/write references for each symbol
/// - Calculates line/column positions
pub(super) fn extract_symbols_from_program(
    program: &oxc_ast::ast::Program<'_>,
    source: &str,
) -> SymbolTable {
    // Build semantic information
    let semantic_ret = SemanticBuilder::new().build(program);

    // Extract the semantic data
    let semantic = semantic_ret.semantic;

    // Get the scoping information which contains the symbol table
    let scoping = semantic.scoping();

    // Pre-allocate the symbol table with the known symbol count
    let mut table = SymbolTable::with_capacity(scoping.symbols_len());
    table.scope_count = scoping.scopes_len();

    // Extract each symbol from Oxc's semantic analysis
    for symbol_id in scoping.symbol_ids() {
        let symbol_flags = scoping.symbol_flags(symbol_id);
        let symbol_name = scoping.symbol_name(symbol_id);
        let symbol_span = scoping.symbol_span(symbol_id);
        let symbol_scope_id = scoping.symbol_scope_id(symbol_id).index() as u32;

        // Determine the kind of symbol (function, class, variable, etc.)
        let kind = determine_symbol_kind(symbol_flags);

        // Calculate line and column from the span
        let (line, column) = get_line_column(source, symbol_span.start);
        let declaration_span = SymbolSpan::new(line, column, symbol_span.start);

        // Create the symbol with initial zero counts
        let mut symbol = Symbol::new(
            symbol_name.to_string(),
            kind,
            declaration_span,
            symbol_scope_id,
        );

        // Count read and write references
        for &reference_id in scoping.get_resolved_reference_ids(symbol_id) {
            let reference = scoping.get_reference(reference_id);
            if reference.is_read() {
                symbol.read_count += 1;
            }
            if reference.is_write() {
                symbol.write_count += 1;
            }
        }

        // Note: is_exported is set to false here and will be updated later
        // during graph building when we analyze export statements

        table.add_symbol(symbol);
    }

    table
}
