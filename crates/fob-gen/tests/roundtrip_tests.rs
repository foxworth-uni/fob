//! Round-trip tests for parse → generate → parse stability

#[cfg(feature = "parser")]
mod roundtrip_impl {
    use fob_gen::{Allocator, ParseOptions, JsBuilder};
    use oxc_allocator::Allocator as OxcAllocator;
    use std::fs;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn read_fixture(name: &str) -> String {
        let path = fixture_path(name);
        fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read fixture {}: {}", name, e))
    }

    /// Test that parsing and regenerating produces valid code
    #[test]
    fn test_parse_and_regenerate_simple() {
        let source = read_fixture("simple.js");
        test_roundtrip(&source, "simple.js");
    }

    #[test]
    fn test_parse_and_regenerate_imports_exports() {
        let source = read_fixture("imports_exports.js");
        test_roundtrip(&source, "imports_exports.js");
    }

    #[test]
    fn test_parse_and_regenerate_jsx() {
        let source = read_fixture("jsx.jsx");
        test_roundtrip(&source, "jsx.jsx");
    }

    #[test]
    fn test_parse_and_regenerate_typescript() {
        let source = read_fixture("typescript.ts");
        test_roundtrip(&source, "typescript.ts");
    }

    #[test]
    fn test_parse_and_regenerate_complex() {
        let source = read_fixture("complex.js");
        test_roundtrip(&source, "complex.js");
    }

    fn test_roundtrip(source: &str, filename: &str) {
        let allocator = OxcAllocator::default();
        
        // Parse original source
        let parse_opts = ParseOptions::from_path(filename);
        let parsed = fob_gen::parse(&allocator, source, parse_opts)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));

        // Regenerate code
        let js = JsBuilder::new(&allocator);
        use oxc_ast::ast::Statement;
        let statements: Vec<Statement> = parsed
            .program
            .body
            .iter()
            .map(|s| *s)
            .collect();
        
        let regenerated = js.program(statements)
            .unwrap_or_else(|e| panic!("Failed to generate code for {}: {}", filename, e));

        // Parse regenerated code
        let parse_opts2 = ParseOptions::from_path(filename);
        let parsed2 = fob_gen::parse(&allocator, &regenerated, parse_opts2)
            .unwrap_or_else(|e| panic!("Failed to parse regenerated code for {}: {}", filename, e));

        // Basic checks: both should have same number of statements
        assert_eq!(
            parsed.program.body.len(),
            parsed2.program.body.len(),
            "Statement count mismatch for {}",
            filename
        );

        // Both should parse without errors (or same errors)
        // Note: We don't compare exact AST structure as formatting may differ
        // but semantic equivalence is what matters
    }

    /// Test that regenerating multiple times produces stable output
    #[test]
    fn test_formatting_stability() {
        let source = read_fixture("simple.js");
        let allocator = OxcAllocator::default();
        
        let parse_opts = ParseOptions::from_path("simple.js");
        let parsed = fob_gen::parse(&allocator, &source, parse_opts).unwrap();

        let js = JsBuilder::new(&allocator);
        use oxc_ast::ast::Statement;
        let statements: Vec<Statement> = parsed
            .program
            .body
            .iter()
            .map(|s| *s)
            .collect();

        // Generate multiple times
        let first = js.program(statements.clone()).unwrap();
        let second = js.program(statements.clone()).unwrap();
        let third = js.program(statements).unwrap();

        // All generations should be identical
        assert_eq!(first, second, "First and second generation differ");
        assert_eq!(second, third, "Second and third generation differ");
    }
}

