//! Unreachable code detection functions.

use super::super::ModuleId;
use super::super::symbol::UnreachableCode;
use super::visitor::UnreachableCodeVisitor as Visitor;

// Note: Parsing is done inline in detect_unreachable_code to avoid
// lifetime issues with the allocator. The detection logic is separated
// into detect_unreachable_with_visitor.

/// Detect unreachable code using AST visitor pattern.
pub(super) fn detect_unreachable_with_visitor(
    program: &oxc_ast::ast::Program<'_>,
    source_text: &str,
    module_id: ModuleId,
) -> Vec<UnreachableCode> {
    // Create visitor to detect unreachable code
    let mut visitor = Visitor {
        source_text,
        module_id,
        unreachable: Vec::new(),
    };

    // Visit the AST to find unreachable code
    use crate::oxc::Visit;
    visitor.visit_program(program);

    visitor.unreachable
}
