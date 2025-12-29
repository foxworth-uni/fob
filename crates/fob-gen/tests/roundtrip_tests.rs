//! Round-trip tests for parse → generate → parse stability

#[cfg(feature = "parser")]
mod roundtrip_impl {
    use fob_gen::{ParseOptions, ProgramBuilder};
    use oxc_allocator::Allocator as OxcAllocator;
    use oxc_codegen;
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
        let source = read_fixture("imports_exports.jsx");
        test_roundtrip(&source, "imports_exports.jsx");
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
        let source = read_fixture("complex.jsx");
        test_roundtrip(&source, "complex.jsx");
    }

    fn test_roundtrip(source: &str, filename: &str) {
        let allocator = OxcAllocator::default();

        // Parse original source
        let parse_opts = ParseOptions::from_path(filename);
        let parsed = fob_gen::parse(&allocator, source, parse_opts)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", filename, e));

        // Regenerate code from parsed AST using oxc_codegen
        let codegen = oxc_codegen::Codegen::new();
        let regenerated = codegen.build(&parsed.program).code;

        // Parse regenerated code with fresh allocator (required for lifetime safety)
        let allocator2 = OxcAllocator::default();
        let parse_opts2 = ParseOptions::from_path(filename);
        let parsed2 = fob_gen::parse(&allocator2, &regenerated, parse_opts2).unwrap_or_else(|e| {
            panic!(
                "Failed to parse regenerated code for {}: {}\nRegenerated:\n{}",
                filename, e, regenerated
            )
        });

        // Basic checks: both should have same number of statements
        assert_eq!(
            parsed.program.body.len(),
            parsed2.program.body.len(),
            "Statement count mismatch for {}\nOriginal: {} statements\nRegenerated: {} statements\nRegenerated code:\n{}",
            filename,
            parsed.program.body.len(),
            parsed2.program.body.len(),
            regenerated
        );
    }

    /// Test that generating code from builder produces stable output
    #[test]
    fn test_formatting_stability() {
        let allocator = OxcAllocator::default();

        let generate_code = |alloc: &OxcAllocator| {
            let mut js = ProgramBuilder::new(alloc);
            let statements = vec![
                js.const_decl("x", js.number(42.0)),
                js.const_decl("y", js.string("hello")),
            ];
            js.extend(statements);
            js.generate(&Default::default()).unwrap()
        };

        // Generate code multiple times
        let first = generate_code(&allocator);
        let second = generate_code(&allocator);
        let third = generate_code(&allocator);

        // All generations should be identical
        assert_eq!(first, second, "First and second generation differ");
        assert_eq!(second, third, "Second and third generation differ");
    }
}
