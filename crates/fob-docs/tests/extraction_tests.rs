use fob_docs::model::SymbolKind;
use fob_docs::{DocsExtractor, ExtractOptions};

#[test]
fn extracts_function_documentation() {
    let source = r#"
        /**
         * Add two numbers together.
         * @param {number} a first value
         * @param {number} b second value
         * @returns {number} sum of a and b
         */
        export function add(a: number, b: number): number {
            return a + b;
        }
    "#;

    let extractor = DocsExtractor::new(ExtractOptions::default());
    let module = extractor
        .extract_from_source("src/math.ts", source)
        .expect("extraction should succeed");

    assert_eq!(module.symbols.len(), 1);
    let symbol = &module.symbols[0];
    assert_eq!(symbol.name, "add");
    assert_eq!(symbol.kind, SymbolKind::Function);
    assert_eq!(symbol.parameters.len(), 2);
    assert_eq!(symbol.summary.as_deref(), Some("Add two numbers together."));
    assert_eq!(symbol.returns.as_deref(), Some("number sum of a and b"));
}

#[test]
fn skips_internal_symbols_by_default() {
    let source = r#"
        /** @internal */
        export function hidden(): void {}
    "#;

    let extractor = DocsExtractor::new(ExtractOptions::default());
    let module = extractor
        .extract_from_source("src/lib.ts", source)
        .expect("extraction should succeed");

    assert!(module.symbols.is_empty());
}

#[test]
fn includes_internal_symbols_when_requested() {
    let source = r#"
        /** @internal */
        export const internalValue = 42;
    "#;

    let extractor = DocsExtractor::new(ExtractOptions {
        include_internal: true,
    });

    let module = extractor
        .extract_from_source("src/lib.ts", source)
        .expect("extraction should succeed");

    assert_eq!(module.symbols.len(), 1);
    assert_eq!(module.symbols[0].name, "internalValue");
}
