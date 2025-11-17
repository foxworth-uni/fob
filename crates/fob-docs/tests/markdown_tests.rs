#![cfg(feature = "markdown")]
use fob_docs::generators::markdown::render_markdown;
use fob_docs::model::{
    Documentation, ExportedSymbol, ModuleDoc, ParameterDoc, SourceLocation, SymbolKind,
};

#[test]
fn renders_markdown_output() {
    let mut module = ModuleDoc::new("src/math.ts");
    module.description = Some("Math utilities.".to_string());

    let mut symbol = ExportedSymbol::new("add", SymbolKind::Function, SourceLocation::new(10, 1));
    symbol.summary = Some("Add two numbers.".to_string());
    symbol.returns = Some("number sum".to_string());
    let mut param = ParameterDoc::new("a");
    param.description = Some("First value.".to_string());
    symbol.parameters.push(param);

    module.symbols.push(symbol);

    let mut documentation = Documentation::default();
    documentation.add_module(module);

    let markdown = render_markdown(&documentation);
    assert!(markdown.contains("# src/math.ts"));
    assert!(markdown.contains("## `add` (function)"));
    assert!(markdown.contains("Add two numbers."));
    assert!(markdown.contains("**Parameters**"));
}
