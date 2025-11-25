//! Semantic analysis engine using Oxc for symbol extraction.
//!
//! This module provides the core functionality for analyzing JavaScript/TypeScript
//! files to extract symbol information, detect unused declarations, and identify
//! unreachable code.

mod analysis;
mod detection;
mod utils;
mod visitor;

use super::symbol::{SymbolTable, UnreachableCode};
use super::{ModuleId, SourceType};
use fob::Result;

use analysis::extract_symbols_from_program;
use detection::detect_unreachable_with_visitor;

/// Analyzes JavaScript/TypeScript source code to extract symbol information.
///
/// This function uses Oxc's semantic analyzer to build a complete symbol table containing
/// all declared symbols (variables, functions, classes, types, etc.) along with their
/// usage statistics.
///
/// # What it does
///
/// - **Symbol Extraction**: Identifies all declarations (let, const, var, function, class, etc.)
/// - **Reference Counting**: Tracks how many times each symbol is read or written
/// - **Scope Analysis**: Maintains scope hierarchy information
/// - **TypeScript Support**: Handles interfaces, type aliases, enums, and other TS constructs
/// - **Symbol Classification**: Categorizes symbols by kind (Variable, Function, Class, etc.)
///
/// # Arguments
///
/// * `source_text` - The JavaScript/TypeScript source code to analyze
/// * `filename` - The filename (used for error reporting and source type detection)
/// * `source_type` - The type of source file (JavaScript, TypeScript, JSX, TSX)
///
/// # Returns
///
/// Returns `Ok(SymbolTable)` containing all symbols found. If parsing fails (syntax errors),
/// returns `Ok(empty_table)` to allow analysis to continue for other files.
///
/// # Error Handling
///
/// This function uses graceful degradation:
/// - Parse errors result in an empty symbol table, not an error
/// - Non-JavaScript files (CSS, JSON, etc.) return empty tables
/// - This ensures that analysis can continue even if some files have issues
///
/// # Examples
///
/// ## Basic usage
///
/// ```rust,ignore
/// use fob::graph::semantic::analyze_symbols;
/// use fob::graph::SourceType;
///
/// let source = r#"
///     const unused = 42;
///     const used = 100;
///     console.log(used);
/// "#;
///
/// let table = analyze_symbols(source, "example.js", SourceType::JavaScript)?;
/// assert_eq!(table.symbols.len(), 2);
///
/// // Check for unused symbols
/// let unused_symbols = table.unused_symbols();
/// assert_eq!(unused_symbols.len(), 1);
/// assert_eq!(unused_symbols[0].name, "unused");
/// ```
///
/// ## TypeScript analysis
///
/// ```rust,ignore
/// let source = r#"
///     interface User {
///         name: string;
///     }
///
///     type UserId = string;
///
///     function getUser(id: UserId): User {
///         return { name: "test" };
///     }
/// "#;
///
/// let table = analyze_symbols(source, "types.ts", SourceType::TypeScript)?;
///
/// // Should find interface, type alias, and function
/// assert!(table.symbols.len() >= 3);
/// ```
///
/// # Performance
///
/// This function parses the source code using Oxc's fast parser and builds semantic
/// information in a single pass. For typical JavaScript files, analysis completes
/// in microseconds.
///
/// # Security
///
/// - All input is validated through Oxc's parser
/// - No code execution occurs - purely static analysis
/// - Safe to use on untrusted input
pub fn analyze_symbols(
    source: &str,
    filename: &str,
    source_type: SourceType,
) -> Result<SymbolTable> {
    // Handle non-JavaScript files
    if !source_type.is_javascript_like() {
        return Ok(SymbolTable::new());
    }

    // Convert our SourceType to Oxc's SourceType
    use utils::convert_source_type;
    let oxc_source_type = convert_source_type(source_type, filename);

    // Create allocator for Oxc's arena-based allocation
    use fob::oxc::{Allocator, Parser};
    let allocator = Allocator::default();

    // Parse the source code
    let parser_return = Parser::new(&allocator, source, oxc_source_type).parse();

    // If there are parse errors, return empty table (graceful degradation)
    if !parser_return.errors.is_empty() {
        return Ok(SymbolTable::new());
    }

    // Extract symbols from the parsed program
    let mut table = extract_symbols_from_program(&parser_return.program, source);

    // Extract class and enum members (which aren't tracked by Oxc's symbol table)
    super::class_enum_extraction::extract_class_and_enum_members(
        &parser_return.program,
        source,
        &mut table,
    );

    // Calculate code quality metrics for functions and classes
    super::quality::calculate_quality_metrics(&parser_return.program, source, &mut table);

    Ok(table)
}

/// Detect unreachable code in a JavaScript/TypeScript file.
///
/// This function uses a simple AST-based approach to detect code that appears
/// after control flow terminators (return, throw, break, continue) in the same
/// block. This is a simplified analysis and does not use full CFG.
///
/// # Arguments
///
/// * `source_text` - The source code to analyze
/// * `filename` - The filename (for reporting)
/// * `source_type` - The type of source file
/// * `module_id` - The module ID for the unreachable code entries
///
/// # Returns
///
/// Returns a vector of `UnreachableCode` entries, one for each unreachable
/// statement detected.
pub fn detect_unreachable_code(
    source_text: &str,
    filename: &str,
    source_type: SourceType,
    module_id: ModuleId,
) -> Result<Vec<UnreachableCode>> {
    // Handle non-JavaScript files
    if !source_type.is_javascript_like() {
        return Ok(Vec::new());
    }

    // Convert our SourceType to Oxc's SourceType
    use utils::convert_source_type;
    let oxc_source_type = convert_source_type(source_type, filename);

    // Create allocator for Oxc's arena-based allocation
    use fob::oxc::{Allocator, Parser};
    let allocator = Allocator::default();

    // Parse the source code
    let parser_return = Parser::new(&allocator, source_text, oxc_source_type).parse();

    // If there are parse errors, return empty (graceful degradation)
    if !parser_return.errors.is_empty() {
        return Ok(Vec::new());
    }

    // Detect unreachable code using visitor
    let unreachable =
        detect_unreachable_with_visitor(&parser_return.program, source_text, module_id);

    Ok(unreachable)
}

#[cfg(test)]
mod tests {
    use super::super::symbol::SymbolKind;
    use super::*;

    #[test]
    fn test_analyze_simple_symbols() {
        let source = r#"
            const used = 42;
            const unused = 100;
            console.log(used);
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        // Should find 'used' and 'unused' variables
        assert!(table.symbols.len() >= 2);

        // Find the 'used' symbol
        let used = table.symbols_by_name("used");
        assert_eq!(used.len(), 1);
        assert!(used[0].read_count > 0, "used should have read references");
    }

    #[test]
    fn test_analyze_functions() {
        let source = r#"
            function usedFunction() {
                return 42;
            }

            function unusedFunction() {
                return 100;
            }

            usedFunction();
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let used_fn = table.symbols_by_name("usedFunction");
        assert_eq!(used_fn.len(), 1);
        assert_eq!(used_fn[0].kind, SymbolKind::Function);
        assert!(used_fn[0].read_count > 0);

        let unused_fn = table.symbols_by_name("unusedFunction");
        assert_eq!(unused_fn.len(), 1);
        assert_eq!(unused_fn[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_analyze_typescript() {
        let source = r#"
            interface User {
                name: string;
            }

            type UserId = string;

            const user: User = { name: "test" };
        "#;

        let table =
            analyze_symbols(source, "test.ts", SourceType::TypeScript).expect("analysis failed");

        // Should find interface, type alias, and variable
        let interface_sym = table.symbols_by_name("User");
        let type_sym = table.symbols_by_name("UserId");
        let var_sym = table.symbols_by_name("user");

        assert_eq!(interface_sym.len(), 1);
        assert_eq!(type_sym.len(), 1);
        assert_eq!(var_sym.len(), 1);
    }

    #[test]
    fn test_graceful_parse_error_handling() {
        let invalid_source = r#"
            const x = {{{{{ invalid syntax
        "#;

        // Should not panic, should return empty table
        let table = analyze_symbols(invalid_source, "invalid.js", SourceType::JavaScript)
            .expect("should handle parse errors gracefully");

        assert!(
            table.is_empty(),
            "should return empty table for invalid syntax"
        );
    }

    #[test]
    fn test_non_javascript_files() {
        let css_content = "body { color: red; }";

        let table = analyze_symbols(css_content, "styles.css", SourceType::Css)
            .expect("should handle non-JS files");

        assert!(table.is_empty(), "should return empty table for CSS files");
    }

    #[test]
    fn test_line_column_calculation() {
        use super::utils::get_line_column;
        let source = "line 1\nline 2\nline 3";

        // Start of file
        assert_eq!(get_line_column(source, 0), (1, 0));

        // Start of line 2 (after first \n)
        assert_eq!(get_line_column(source, 7), (2, 0));

        // Start of line 3
        assert_eq!(get_line_column(source, 14), (3, 0));
    }

    #[test]
    fn test_symbol_spans() {
        let source = r#"
const x = 1;
function f() {}
"#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        // All symbols should have valid spans
        for symbol in &table.symbols {
            assert!(symbol.declaration_span.line > 0, "line should be positive");
        }
    }

    #[test]
    fn test_scope_tracking() {
        let source = r#"
            const global = 1;
            function outer() {
                const local = 2;
                function inner() {
                    const nested = 3;
                }
            }
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        // Should track multiple scopes
        assert!(table.scope_count > 1, "should detect multiple scopes");
    }

    #[test]
    fn test_unused_variable_detection() {
        let source = r#"
            function example() {
                const used = 42;
                const unused = 100;
                return used;
            }
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        // Find used and unused symbols
        let used = table.symbols_by_name("used");
        let unused = table.symbols_by_name("unused");

        assert_eq!(used.len(), 1, "should find 'used' symbol");
        assert_eq!(unused.len(), 1, "should find 'unused' symbol");

        // Used should have reads
        assert!(used[0].read_count > 0, "used should be read");

        // Unused should have no reads
        assert_eq!(unused[0].read_count, 0, "unused should not be read");
    }

    #[test]
    fn test_used_function_has_reads() {
        let source = r#"
            function helper() { return 42; }
            const x = helper();
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let helper = table.symbols_by_name("helper");
        assert_eq!(helper.len(), 1, "should find 'helper' function");
        assert!(helper[0].read_count > 0, "helper() should be called");
    }

    #[test]
    fn test_typescript_types() {
        let source = r#"
            interface User {
                name: string;
            }

            type UserId = string;

            const user: User = { name: "test" };
        "#;

        let table =
            analyze_symbols(source, "test.ts", SourceType::TypeScript).expect("analysis failed");

        // Check for interface
        let interface_sym = table.symbols_by_name("User");
        assert_eq!(interface_sym.len(), 1);
        assert_eq!(interface_sym[0].kind, SymbolKind::Interface);

        // Check for type alias
        let type_sym = table.symbols_by_name("UserId");
        assert_eq!(type_sym.len(), 1);
        assert_eq!(type_sym[0].kind, SymbolKind::TypeAlias);

        // Check for variable
        let var_sym = table.symbols_by_name("user");
        assert_eq!(var_sym.len(), 1);
        assert_eq!(var_sym[0].kind, SymbolKind::Variable);
    }

    #[test]
    fn test_unreachable_after_return() {
        let source = r#"
            function example() {
                return true;
                console.log('unreachable');
            }
        "#;

        let module_id = ModuleId::new("test.js").expect("valid module id");
        let unreachable =
            detect_unreachable_code(source, "test.js", SourceType::JavaScript, module_id)
                .expect("detection failed");

        assert!(!unreachable.is_empty(), "should detect unreachable code");

        // Check description
        let desc = &unreachable[0].description;
        assert!(desc.contains("return"), "should mention return statement");
    }

    #[test]
    fn test_unreachable_after_throw() {
        let source = r#"
            function example() {
                throw new Error('test');
                console.log('unreachable');
            }
        "#;

        let module_id = ModuleId::new("test.js").expect("valid module id");
        let unreachable =
            detect_unreachable_code(source, "test.js", SourceType::JavaScript, module_id)
                .expect("detection failed");

        assert!(
            !unreachable.is_empty(),
            "should detect unreachable code after throw"
        );
    }

    #[test]
    fn test_no_unreachable_when_none() {
        let source = r#"
            function example() {
                console.log('reachable');
                return true;
            }
        "#;

        let module_id = ModuleId::new("test.js").expect("valid module id");
        let unreachable =
            detect_unreachable_code(source, "test.js", SourceType::JavaScript, module_id)
                .expect("detection failed");

        assert_eq!(
            unreachable.len(),
            0,
            "should not detect unreachable code when none exists"
        );
    }

    #[test]
    fn test_class_symbol_kind() {
        let source = r#"
            class MyClass {
                method() {}
            }
        "#;

        let table =
            analyze_symbols(source, "test.js", SourceType::JavaScript).expect("analysis failed");

        let class_sym = table.symbols_by_name("MyClass");
        assert_eq!(class_sym.len(), 1);
        assert_eq!(class_sym[0].kind, SymbolKind::Class);
    }
}
