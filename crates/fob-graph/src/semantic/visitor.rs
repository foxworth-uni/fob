//! AST visitor implementations for semantic analysis.

use fob_core::oxc::{GetSpan, Visit, ast::Statement};
use oxc_ast_visit::walk;

use super::super::ModuleId;
use super::super::symbol::{SymbolSpan, UnreachableCode};
use super::utils::get_line_column;

/// AST visitor that detects unreachable code.
pub(super) struct UnreachableCodeVisitor<'a> {
    pub(super) source_text: &'a str,
    pub(super) module_id: ModuleId,
    pub(super) unreachable: Vec<UnreachableCode>,
}

impl<'a> UnreachableCodeVisitor<'a> {
    /// Check if a statement is a control flow terminator (returns, throws, etc.)
    fn is_terminator(stmt: &Statement) -> bool {
        matches!(
            stmt,
            Statement::ReturnStatement(_)
                | Statement::ThrowStatement(_)
                | Statement::BreakStatement(_)
                | Statement::ContinueStatement(_)
        )
    }

    /// Mark unreachable code in a block of statements
    pub(super) fn check_block_statements(&mut self, statements: &[Statement]) {
        let mut found_terminator = false;
        let mut terminator_kind = None;

        for stmt in statements {
            if found_terminator {
                // This statement comes after a terminator - it's unreachable
                let span = stmt.span();
                let (line, column) = get_line_column(self.source_text, span.start);

                let description = format!(
                    "Code is unreachable after {} statement",
                    terminator_kind.unwrap_or("control flow")
                );

                self.unreachable.push(UnreachableCode {
                    module_id: self.module_id.clone(),
                    description,
                    span: SymbolSpan::new(line, column, span.start),
                });

                // Note: We only report the first unreachable statement per block
                // to avoid noise. All subsequent statements are also unreachable.
                break;
            }

            if Self::is_terminator(stmt) {
                found_terminator = true;
                terminator_kind = match stmt {
                    Statement::ReturnStatement(_) => Some("return"),
                    Statement::ThrowStatement(_) => Some("throw"),
                    Statement::BreakStatement(_) => Some("break"),
                    Statement::ContinueStatement(_) => Some("continue"),
                    _ => None,
                };
            }
        }
    }
}

impl<'a, 'ast> Visit<'ast> for UnreachableCodeVisitor<'a> {
    fn visit_statements(&mut self, stmts: &oxc_allocator::Vec<'ast, Statement<'ast>>) {
        self.check_block_statements(stmts);
        walk::walk_statements(self, stmts);
    }

    fn visit_function_body(&mut self, body: &oxc_ast::ast::FunctionBody<'ast>) {
        self.check_block_statements(&body.statements);
        walk::walk_function_body(self, body);
    }
}
