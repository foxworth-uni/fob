//! Metric calculation logic for code quality analysis.

use fob::oxc::ast::{Class, ClassElement, Function, Statement};
use std::collections::HashMap;

use crate::symbol::CodeQualityMetadata;

/// AST visitor for calculating code quality metrics
pub(super) struct QualityCalculator<'a> {
    pub(super) source_text: &'a str,
    pub(super) metrics: HashMap<String, CodeQualityMetadata>,
}

impl<'a> QualityCalculator<'a> {
    /// Calculate line count from start to end position in source
    pub(super) fn calculate_line_count(&self, start: u32, end: u32) -> usize {
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
    pub(super) fn calculate_function_metrics(&self, func: &Function) -> CodeQualityMetadata {
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
    pub(super) fn calculate_class_metrics(&self, class: &Class) -> CodeQualityMetadata {
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
    pub(super) fn calculate_complexity(&self, statements: &oxc_allocator::Vec<Statement>) -> usize {
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
    pub(super) fn calculate_nesting_depth(
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
    pub(super) fn count_returns(&self, statements: &oxc_allocator::Vec<Statement>) -> usize {
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

