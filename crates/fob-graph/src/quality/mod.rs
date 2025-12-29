//! Code quality metrics extraction from AST.
//!
//! This module provides functionality for calculating code quality metrics
//! (line count, parameter count, complexity, etc.) from JavaScript/TypeScript
//! AST nodes and attaching them to symbols.

mod metrics;
mod visitor;

// Use re-exported OXC types from the fob crate for version consistency
use crate::oxc::ast::Program;
use std::collections::HashMap;

use crate::symbol::{SymbolMetadata, SymbolTable};

use metrics::QualityCalculator;

/// Calculate code quality metrics for functions and classes and attach to symbols
///
/// This uses an AST visitor to analyze all function and class declarations,
/// calculating metrics like line count, parameter count, complexity, etc.
pub fn calculate_quality_metrics(program: &Program, source_text: &str, table: &mut SymbolTable) {
    let mut calculator = QualityCalculator {
        source_text,
        metrics: HashMap::new(),
    };

    use crate::oxc::Visit;
    calculator.visit_program(program);

    // Attach calculated metrics to matching symbols in the table
    for symbol in &mut table.symbols {
        if let Some(metadata) = calculator.metrics.remove(&symbol.name) {
            symbol.metadata = SymbolMetadata::CodeQuality(metadata);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::SourceType;
    use crate::semantic::analyze_symbols;

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

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

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

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

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

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

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

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let func_symbols = table.symbols_by_name("complexFunction");
        assert_eq!(func_symbols.len(), 1);

        let metadata = func_symbols[0].code_quality_metadata().unwrap();
        assert!(metadata.complexity.is_some());
        // Should have complexity > 1 (has if statements)
        assert!(metadata.complexity.unwrap() > 1);
    }
}
