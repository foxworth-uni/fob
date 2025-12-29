//! Symbol analysis functions for extracting symbol information from source code.

use super::super::symbol::{QualifiedReference, Symbol, SymbolSpan, SymbolTable};
use super::utils::{LineIndex, determine_symbol_kind};
use crate::oxc::SemanticBuilder;
use oxc_ast::AstKind;
use oxc_span::GetSpan;

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

    // Pre-calculate line offsets for O(1) lookups
    let line_index = LineIndex::new(source);

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
        let (line, column) = line_index.get_line_column(symbol_span.start, source);
        let declaration_span = SymbolSpan::new(line, column, symbol_span.start);

        // Create the symbol with initial zero counts
        let mut symbol = Symbol::new(
            symbol_name.to_string(),
            kind,
            declaration_span,
            symbol_scope_id,
        );

        // Count read and write references and track qualified usages
        for &reference_id in scoping.get_resolved_reference_ids(symbol_id) {
            let reference = scoping.get_reference(reference_id);
            // Count both runtime reads AND type-only references as "reads"
            // Type references (e.g., `const x: User`) have is_type()=true but is_read()=false
            // For dead code detection, type usage IS meaningful usage
            if reference.is_read() || reference.is_type() {
                symbol.read_count += 1;
            }
            if reference.is_write() {
                symbol.write_count += 1;
            }

            // Check for qualified member access (e.g. React.ComponentProps)
            // We only care about reads that might be used for types or namespaces
            if reference.is_read() || reference.is_type() {
                let nodes = semantic.nodes();
                let mut member_path = Vec::new();
                let mut is_type = reference.is_type();

                let mut last_node_id = reference.node_id();
                for parent_id in nodes.ancestor_ids(last_node_id) {
                    let parent_node = nodes.get_node(parent_id);
                    match parent_node.kind() {
                        AstKind::StaticMemberExpression(expr) => {
                            // Check if we are the object (left side)
                            if expr.object.span() == nodes.get_node(last_node_id).kind().span() {
                                member_path.push(expr.property.name.to_string());
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        AstKind::ComputedMemberExpression(expr) => {
                            // Check if we are the object (left side)
                            if expr.object.span() == nodes.get_node(last_node_id).kind().span() {
                                // For computed refs like Lib['prop'], we generally stop or mark as dynamic
                                // usage. Here we just stop tracking chain.
                            }
                        }
                        AstKind::TSQualifiedName(name) => {
                            // Check if we are the left side
                            // e.g. React.ComponentProps -> React is left, ComponentProps is right
                            let current_span = nodes.get_node(last_node_id).kind().span();
                            if name.left.span() == current_span {
                                member_path.push(name.right.name.to_string());
                                last_node_id = parent_id;
                                is_type = true; // TSQualifiedName is always a type position
                                continue;
                            }
                        }
                        AstKind::JSXMemberExpression(expr) => {
                            // Check if we are the object (left side)
                            if expr.object.span() == nodes.get_node(last_node_id).kind().span() {
                                member_path.push(expr.property.name.to_string());
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        AstKind::ChainExpression(_) => {
                            // ChainExpression wraps the optional expression (e.g. (a?.b))
                            // We just need to step through it to the parent
                            last_node_id = parent_id;
                            continue;
                        }
                        AstKind::ParenthesizedExpression(_) => {
                            // Just step through parentheses
                            last_node_id = parent_id;
                            continue;
                        }
                        AstKind::TSAsExpression(expr) => {
                            if expr.expression.span() == nodes.get_node(last_node_id).kind().span()
                            {
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        AstKind::TSSatisfiesExpression(expr) => {
                            if expr.expression.span() == nodes.get_node(last_node_id).kind().span()
                            {
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        AstKind::TSNonNullExpression(expr) => {
                            if expr.expression.span() == nodes.get_node(last_node_id).kind().span()
                            {
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        AstKind::TSInstantiationExpression(expr) => {
                            if expr.expression.span() == nodes.get_node(last_node_id).kind().span()
                            {
                                last_node_id = parent_id;
                                continue;
                            }
                        }
                        _ => {}
                    }
                    break;
                }

                if !member_path.is_empty() {
                    let span = nodes.get_node(last_node_id).kind().span();
                    let (line, column) = line_index.get_line_column(span.start, source);

                    symbol.qualified_references.push(QualifiedReference {
                        member_path,
                        is_type,
                        span: SymbolSpan::new(line, column, span.start),
                    });
                }
            }
        }

        // Note: is_exported is set to false here and will be updated later
        // during graph building when we analyze export statements

        table.add_symbol(symbol);
    }

    table
}
