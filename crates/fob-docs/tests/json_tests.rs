#![cfg(feature = "json")]

use fob_docs::generators::json::render_json;
use fob_docs::model::{Documentation, ExportedSymbol, ModuleDoc, SourceLocation, SymbolKind};

#[test]
fn renders_json_output() {
    let mut module = ModuleDoc::new("src/math.ts");
    module.symbols.push(ExportedSymbol::new(
        "add",
        SymbolKind::Function,
        SourceLocation::new(5, 1),
    ));

    let mut documentation = Documentation::default();
    documentation.add_module(module);

    let json = render_json(&documentation).expect("should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

    assert_eq!(value["documentation"]["modules"][0]["path"], "src/math.ts");
}
