//! Code quality metrics extraction from AST.
//!
//! This module provides functionality for calculating code quality metrics
//! (line count, parameter count, complexity, etc.) from JavaScript/TypeScript
//! AST nodes and attaching them to symbols.

use oxc_ast::ast::{Class, ClassElement, Function, Program, Statement};
use oxc_ast_visit::{walk, Visit};
use oxc_semantic::ScopeFlags;
use std::collections::HashMap;

use super::symbol::{CodeQualityMetadata, SymbolMetadata, SymbolTable};

/// Calculate code quality metrics for functions and classes and attach to symbols
///
/// This uses an AST visitor to analyze all function and class declarations,
/// calculating metrics like line count, parameter count, complexity, etc.
pub fn calculate_quality_metrics(program: &Program, source_text: &str, table: &mut SymbolTable) {
    let mut calculator = QualityCalculator {
        source_text,
        metrics: HashMap::new(),
    };

    calculator.visit_program(program);

    // Attach calculated metrics to matching symbols in the table
    for symbol in &mut table.symbols {
        if let Some(metadata) = calculator.metrics.remove(&symbol.name) {
            symbol.metadata = SymbolMetadata::CodeQuality(metadata);
        }
    }
}

/// AST visitor for calculating code quality metrics
struct QualityCalculator<'a> {
    source_text: &'a str,
    metrics: HashMap<String, CodeQualityMetadata>,
}

impl<'a> QualityCalculator<'a> {
    /// Calculate line count from start to end position in source
    fn calculate_line_count(&self, start: u32, end: u32) -> usize {
        let start_idx = start as usize;
        let end_idx = end as usize;

        if start_idx >= end_idx || end_idx > self.source_text.len() {
            return 1;
        }

        // Count newlines in the range
        let newlines = self.source_text[start_idx..end_idx]
            .chars()
            .filter(|&c| c == '\n')
            .count();

        // Line count is newlines + 1 (since first line doesn't start with newline)
        newlines + 1
    }

    /// Calculate metrics for a function declaration
    fn calculate_function_metrics(&self, func: &Function) -> CodeQualityMetadata {
        let line_count = if let Some(body) = &func.body {
            self.calculate_line_count(body.span.start, body.span.end)
        } else {
            1 // Function declaration without body (e.g., ambient declaration)
        };

        let parameter_count = func.params.items.len();

        // Calculate complexity (simplified McCabe's cyclomatic complexity)
        let complexity = if let Some(body) = &func.body {
            self.calculate_complexity(&body.statements)
        } else {
            1 // Default complexity
        };

        // Calculate maximum nesting depth
        let max_nesting = if let Some(body) = &func.body {
            self.calculate_nesting_depth(&body.statements, 0)
        } else {
            0
        };

        // Count return statements
        let return_count = if let Some(body) = &func.body {
            self.count_returns(&body.statements)
        } else {
            0
        };

        CodeQualityMetadata::for_function(
            Some(line_count),
            Some(parameter_count),
            Some(complexity),
            Some(max_nesting),
            Some(return_count),
        )
    }

    /// Calculate metrics for a class declaration
    fn calculate_class_metrics(&self, class: &Class) -> CodeQualityMetadata {
        let line_count = self.calculate_line_count(class.span.start, class.span.end);

        // Count methods in the class
        let method_count = class
            .body
            .body
            .iter()
            .filter(|element| matches!(element, ClassElement::MethodDefinition(_)))
            .count();

        // Count fields (properties) in the class
        let field_count = class
            .body
            .body
            .iter()
            .filter(|element| {
                matches!(
                    element,
                    ClassElement::PropertyDefinition(_) | ClassElement::AccessorProperty(_)
                )
            })
            .count();

        CodeQualityMetadata::for_class(Some(line_count), Some(method_count), Some(field_count))
    }

    /// Calculate cyclomatic complexity (simplified version)
    ///
    /// Counts decision points: if, while, for, case, catch, &&, ||, ?:
    /// Starts at 1 (one path through the code)
    fn calculate_complexity(&self, statements: &oxc_allocator::Vec<Statement>) -> usize {
        let mut complexity = 1;

        for stmt in statements {
            complexity += self.count_decision_points_in_statement(stmt);
        }

        complexity
    }

    /// Count decision points in a single statement
    fn count_decision_points_in_statement(&self, stmt: &Statement) -> usize {
        let mut count = 0;

        match stmt {
            Statement::IfStatement(if_stmt) => {
                count += 1; // The if itself

                // Check consequent
                if let Statement::BlockStatement(block) = &if_stmt.consequent {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&if_stmt.consequent);
                }

                // Check alternate (else/else if)
                if let Some(alternate) = &if_stmt.alternate {
                    if let Statement::BlockStatement(block) = alternate {
                        count += self.calculate_complexity(&block.body).saturating_sub(1);
                    } else {
                        count += self.count_decision_points_in_statement(alternate);
                    }
                }
            }
            Statement::WhileStatement(while_stmt) => {
                count += 1;
                if let Statement::BlockStatement(block) = &while_stmt.body {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&while_stmt.body);
                }
            }
            Statement::DoWhileStatement(do_while) => {
                count += 1;
                if let Statement::BlockStatement(block) = &do_while.body {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&do_while.body);
                }
            }
            Statement::ForStatement(for_stmt) => {
                count += 1;
                if let Statement::BlockStatement(block) = &for_stmt.body {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&for_stmt.body);
                }
            }
            Statement::ForInStatement(for_in) => {
                count += 1;
                if let Statement::BlockStatement(block) = &for_in.body {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&for_in.body);
                }
            }
            Statement::ForOfStatement(for_of) => {
                count += 1;
                if let Statement::BlockStatement(block) = &for_of.body {
                    count += self.calculate_complexity(&block.body).saturating_sub(1);
                } else {
                    count += self.count_decision_points_in_statement(&for_of.body);
                }
            }
            Statement::SwitchStatement(switch) => {
                // Each case adds a decision point
                count += switch.cases.len();
                for case in &switch.cases {
                    count += self
                        .calculate_complexity(&case.consequent)
                        .saturating_sub(1);
                }
            }
            Statement::TryStatement(try_stmt) => {
                if try_stmt.handler.is_some() {
                    count += 1; // catch adds a decision point
                }
                count += self
                    .calculate_complexity(&try_stmt.block.body)
                    .saturating_sub(1);
                if let Some(handler) = &try_stmt.handler {
                    count += self
                        .calculate_complexity(&handler.body.body)
                        .saturating_sub(1);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    count += self.calculate_complexity(&finalizer.body).saturating_sub(1);
                }
            }
            Statement::BlockStatement(block) => {
                count += self.calculate_complexity(&block.body).saturating_sub(1);
            }
            _ => {}
        }

        count
    }

    /// Calculate maximum nesting depth in a block of statements
    fn calculate_nesting_depth(
        &self,
        statements: &oxc_allocator::Vec<Statement>,
        current_depth: usize,
    ) -> usize {
        let mut max_depth = current_depth;

        for stmt in statements {
            let depth = self.calculate_nesting_for_statement(stmt, current_depth);
            max_depth = max_depth.max(depth);
        }

        max_depth
    }

    /// Calculate nesting depth for a single statement
    fn calculate_nesting_for_statement(&self, stmt: &Statement, current_depth: usize) -> usize {
        let next_depth = current_depth + 1;

        match stmt {
            Statement::IfStatement(if_stmt) => {
                let mut max = next_depth;

                if let Statement::BlockStatement(block) = &if_stmt.consequent {
                    max = max.max(self.calculate_nesting_depth(&block.body, next_depth));
                } else {
                    max = max
                        .max(self.calculate_nesting_for_statement(&if_stmt.consequent, next_depth));
                }

                if let Some(alternate) = &if_stmt.alternate {
                    if let Statement::BlockStatement(block) = alternate {
                        max = max.max(self.calculate_nesting_depth(&block.body, next_depth));
                    } else {
                        max = max.max(self.calculate_nesting_for_statement(alternate, next_depth));
                    }
                }

                max
            }
            Statement::WhileStatement(while_stmt) => {
                if let Statement::BlockStatement(block) = &while_stmt.body {
                    self.calculate_nesting_depth(&block.body, next_depth)
                } else {
                    self.calculate_nesting_for_statement(&while_stmt.body, next_depth)
                }
            }
            Statement::DoWhileStatement(do_while) => {
                if let Statement::BlockStatement(block) = &do_while.body {
                    self.calculate_nesting_depth(&block.body, next_depth)
                } else {
                    self.calculate_nesting_for_statement(&do_while.body, next_depth)
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Statement::BlockStatement(block) = &for_stmt.body {
                    self.calculate_nesting_depth(&block.body, next_depth)
                } else {
                    self.calculate_nesting_for_statement(&for_stmt.body, next_depth)
                }
            }
            Statement::ForInStatement(for_in) => {
                if let Statement::BlockStatement(block) = &for_in.body {
                    self.calculate_nesting_depth(&block.body, next_depth)
                } else {
                    self.calculate_nesting_for_statement(&for_in.body, next_depth)
                }
            }
            Statement::ForOfStatement(for_of) => {
                if let Statement::BlockStatement(block) = &for_of.body {
                    self.calculate_nesting_depth(&block.body, next_depth)
                } else {
                    self.calculate_nesting_for_statement(&for_of.body, next_depth)
                }
            }
            Statement::SwitchStatement(switch) => {
                let mut max = next_depth;
                for case in &switch.cases {
                    max = max.max(self.calculate_nesting_depth(&case.consequent, next_depth));
                }
                max
            }
            Statement::TryStatement(try_stmt) => {
                let mut max = self.calculate_nesting_depth(&try_stmt.block.body, next_depth);
                if let Some(handler) = &try_stmt.handler {
                    max = max.max(self.calculate_nesting_depth(&handler.body.body, next_depth));
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    max = max.max(self.calculate_nesting_depth(&finalizer.body, next_depth));
                }
                max
            }
            Statement::BlockStatement(block) => {
                self.calculate_nesting_depth(&block.body, next_depth)
            }
            _ => current_depth,
        }
    }

    /// Count return statements in a block
    fn count_returns(&self, statements: &oxc_allocator::Vec<Statement>) -> usize {
        let mut count = 0;

        for stmt in statements {
            count += self.count_returns_in_statement(stmt);
        }

        count
    }

    /// Count return statements in a single statement (recursively)
    fn count_returns_in_statement(&self, stmt: &Statement) -> usize {
        match stmt {
            Statement::ReturnStatement(_) => 1,
            Statement::IfStatement(if_stmt) => {
                let mut count = 0;
                if let Statement::BlockStatement(block) = &if_stmt.consequent {
                    count += self.count_returns(&block.body);
                } else {
                    count += self.count_returns_in_statement(&if_stmt.consequent);
                }
                if let Some(alternate) = &if_stmt.alternate {
                    if let Statement::BlockStatement(block) = alternate {
                        count += self.count_returns(&block.body);
                    } else {
                        count += self.count_returns_in_statement(alternate);
                    }
                }
                count
            }
            Statement::WhileStatement(while_stmt) => {
                if let Statement::BlockStatement(block) = &while_stmt.body {
                    self.count_returns(&block.body)
                } else {
                    self.count_returns_in_statement(&while_stmt.body)
                }
            }
            Statement::DoWhileStatement(do_while) => {
                if let Statement::BlockStatement(block) = &do_while.body {
                    self.count_returns(&block.body)
                } else {
                    self.count_returns_in_statement(&do_while.body)
                }
            }
            Statement::ForStatement(for_stmt) => {
                if let Statement::BlockStatement(block) = &for_stmt.body {
                    self.count_returns(&block.body)
                } else {
                    self.count_returns_in_statement(&for_stmt.body)
                }
            }
            Statement::ForInStatement(for_in) => {
                if let Statement::BlockStatement(block) = &for_in.body {
                    self.count_returns(&block.body)
                } else {
                    self.count_returns_in_statement(&for_in.body)
                }
            }
            Statement::ForOfStatement(for_of) => {
                if let Statement::BlockStatement(block) = &for_of.body {
                    self.count_returns(&block.body)
                } else {
                    self.count_returns_in_statement(&for_of.body)
                }
            }
            Statement::SwitchStatement(switch) => {
                let mut count = 0;
                for case in &switch.cases {
                    count += self.count_returns(&case.consequent);
                }
                count
            }
            Statement::TryStatement(try_stmt) => {
                let mut count = self.count_returns(&try_stmt.block.body);
                if let Some(handler) = &try_stmt.handler {
                    count += self.count_returns(&handler.body.body);
                }
                if let Some(finalizer) = &try_stmt.finalizer {
                    count += self.count_returns(&finalizer.body);
                }
                count
            }
            Statement::BlockStatement(block) => self.count_returns(&block.body),
            _ => 0,
        }
    }
}

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
        // TODO: Consider tracking arrow functions by their assignment location
        walk::walk_arrow_function_expression(self, expr);
    }

    /// Visit class declarations
    fn visit_class(&mut self, class: &Class<'ast>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::semantic::analyze_symbols;
    use crate::graph::SourceType;

    #[test]
    fn test_calculate_function_line_count() {
        let source = r#"
function longFunction() {
    let a = 1;
    let b = 2;
    let c = 3;
    return a + b + c;
}
        "#;

        let mut table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        // Parse again to get the program for quality metrics
        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::default();
        let ret = oxc_parser::Parser::new(&allocator, source, source_type).parse();

        calculate_quality_metrics(&ret.program, source, &mut table);

        // Find the function symbol
        let func_symbols = table.symbols_by_name("longFunction");
        assert_eq!(func_symbols.len(), 1);

        // Check that metadata was attached
        let metadata = func_symbols[0].code_quality_metadata();
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert!(metadata.line_count.is_some());
        assert!(metadata.line_count.unwrap() >= 5); // Should be around 5-6 lines
    }

    #[test]
    fn test_calculate_parameter_count() {
        let source = r#"
function manyParams(a, b, c, d, e) {
    return a + b + c + d + e;
}
        "#;

        let mut table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::default();
        let ret = oxc_parser::Parser::new(&allocator, source, source_type).parse();

        calculate_quality_metrics(&ret.program, source, &mut table);

        let func_symbols = table.symbols_by_name("manyParams");
        assert_eq!(func_symbols.len(), 1);

        let metadata = func_symbols[0].code_quality_metadata().unwrap();
        assert_eq!(metadata.parameter_count, Some(5));
    }

    #[test]
    fn test_calculate_class_metrics() {
        let source = r#"
class MyClass {
    field1 = 1;
    field2 = 2;
    
    method1() {}
    method2() {}
    method3() {}
}
        "#;

        let mut table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::default();
        let ret = oxc_parser::Parser::new(&allocator, source, source_type).parse();

        calculate_quality_metrics(&ret.program, source, &mut table);

        let class_symbols = table.symbols_by_name("MyClass");
        assert_eq!(class_symbols.len(), 1);

        let metadata = class_symbols[0].code_quality_metadata().unwrap();
        assert_eq!(metadata.method_count, Some(3));
        assert_eq!(metadata.field_count, Some(2));
    }

    #[test]
    fn test_calculate_complexity() {
        let source = r#"
function complexFunction(x) {
    if (x > 0) {
        if (x < 10) {
            return 1;
        } else {
            return 2;
        }
    } else {
        return 3;
    }
}
        "#;

        let mut table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::default();
        let ret = oxc_parser::Parser::new(&allocator, source, source_type).parse();

        calculate_quality_metrics(&ret.program, source, &mut table);

        let func_symbols = table.symbols_by_name("complexFunction");
        assert_eq!(func_symbols.len(), 1);

        let metadata = func_symbols[0].code_quality_metadata().unwrap();
        assert!(metadata.complexity.is_some());
        // Should have complexity > 1 (has if statements)
        assert!(metadata.complexity.unwrap() > 1);
    }
}
