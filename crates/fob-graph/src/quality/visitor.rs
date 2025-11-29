//! AST visitor implementations for code quality analysis.

use crate::oxc::{ScopeFlags, Visit, ast::Function};
use oxc_ast_visit::walk;

use super::QualityCalculator;

impl<'a, 'ast> Visit<'ast> for QualityCalculator<'a> {
    /// Visit function declarations and expressions
    fn visit_function(&mut self, func: &Function<'ast>, flags: ScopeFlags) {
        // Only process named functions (we need a name to match with symbol table)
        if let Some(id) = &func.id {
            let name = id.name.as_str().to_string();
            let metadata = self.calculate_function_metrics(func);
            self.metrics.insert(name, metadata);
        }

        // Continue visiting nested functions
        walk::walk_function(self, func, flags);
    }

    /// Visit arrow function expressions
    fn visit_arrow_function_expression(
        &mut self,
        expr: &oxc_ast::ast::ArrowFunctionExpression<'ast>,
    ) {
        // Arrow functions are typically assigned to variables, but we can't easily
        // get the variable name here. The symbol table will already have the variable
        // entry from the semantic analysis.
        // For now, we skip arrow functions in metrics calculation.
        //
        // Note: Arrow function tracking is deferred because:
        // 1. The symbol table already tracks arrow functions as variables (when assigned)
        // 2. Tracking standalone arrow functions would require AST parent traversal
        // 3. The current approach covers the common case (arrow functions assigned to variables)
        // 4. Full tracking would add complexity without significant value for code quality metrics
        //
        // If needed in the future, consider:
        // - Traversing AST parents to find assignment context
        // - Or tracking arrow functions separately during semantic analysis
        walk::walk_arrow_function_expression(self, expr);
    }

    /// Visit class declarations
    fn visit_class(&mut self, class: &crate::oxc::ast::Class<'ast>) {
        // Only process named classes
        if let Some(id) = &class.id {
            let name = id.name.as_str().to_string();
            let metadata = self.calculate_class_metrics(class);
            self.metrics.insert(name, metadata);
        }

        // Continue visiting class members (for nested classes/functions)
        walk::walk_class(self, class);
    }
}
