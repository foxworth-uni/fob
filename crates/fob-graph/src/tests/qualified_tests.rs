use super::super::semantic::analyze_symbols;
use crate::SourceType;

fn analyze(source: &str) -> super::super::symbol::SymbolTable {
    analyze_symbols(source, "test.tsx", SourceType::Tsx).expect("analysis failed")
}

#[test]
fn tracks_qualified_type_references() {
    let source = r#"
    import React from 'react';
    type P = React.ComponentProps<'div'>;
    "#;

    let table = analyze(source);
    let react_symbol = table
        .symbols
        .iter()
        .find(|s| s.name == "React")
        .expect("React symbol not found");

    assert!(
        !react_symbol.qualified_references.is_empty(),
        "Should track qualified references"
    );
    let ref0 = &react_symbol.qualified_references[0];

    assert_eq!(ref0.member_path, vec!["ComponentProps"]);
    assert!(ref0.is_type, "Should be marked as type usage");
}

#[test]
fn tracks_qualified_value_references() {
    let source = r#"
    import ReactDOM from 'react-dom';
    ReactDOM.render();
    "#;

    let table = analyze(source);
    let react_dom = table
        .symbols
        .iter()
        .find(|s| s.name == "ReactDOM")
        .expect("ReactDOM symbol not found");

    assert!(!react_dom.qualified_references.is_empty());
    let ref0 = &react_dom.qualified_references[0];

    assert_eq!(ref0.member_path, vec!["render"]);
    assert!(!ref0.is_type, "Should be marked as value usage");
}

#[test]
fn tracks_deeply_nested_chains() {
    let source = r#"
    import UI from 'ui-lib';
    const Theme = UI.Theme.Dark;
    "#;

    let table = analyze(source);
    let ui = table
        .symbols
        .iter()
        .find(|s| s.name == "UI")
        .expect("UI symbol not found");

    assert!(!ui.qualified_references.is_empty());
    let ref0 = &ui.qualified_references[0];

    assert_eq!(ref0.member_path, vec!["Theme", "Dark"]);
}

#[test]
fn tracks_mixed_usage() {
    let source = r#"
    import * as Lib from 'lib';
    const val = Lib.value;
    type T = Lib.Type;
    "#;

    let table = analyze(source);
    let lib = table
        .symbols
        .iter()
        .find(|s| s.name == "Lib")
        .expect("Lib symbol not found");

    assert_eq!(lib.qualified_references.len(), 2);

    // Convert to simplified strings for easier checking since order might vary structurally but usually sequential
    let refs: Vec<String> = lib
        .qualified_references
        .iter()
        .map(|r| {
            format!(
                "{}:{}",
                r.member_path.join("."),
                if r.is_type { "type" } else { "value" }
            )
        })
        .collect();

    assert!(refs.contains(&"value:value".to_string()));
    assert!(refs.contains(&"Type:type".to_string()));
}

#[test]
fn tracks_jsx_namespace_member() {
    let source = r#"
    import Lib from 'lib';
    const el = <Lib.Component />;
    "#;

    let table = analyze(source);
    let lib = table
        .symbols
        .iter()
        .find(|s| s.name == "Lib")
        .expect("Lib symbol not found");

    assert!(
        !lib.qualified_references.is_empty(),
        "Failed to track JSX member expression Lib.Component"
    );
    if !lib.qualified_references.is_empty() {
        assert_eq!(lib.qualified_references[0].member_path, vec!["Component"]);
    }
}

#[test]
fn tracks_optional_chaining_interruption() {
    let source = r#"
    import Lib from 'lib';
    // (Lib?.prop).nested
    const val = (Lib?.prop).nested;
    "#;

    let table = analyze(source);
    let lib = table
        .symbols
        .iter()
        .find(|s| s.name == "Lib")
        .expect("Lib symbol not found");

    let has_nested = lib
        .qualified_references
        .iter()
        .any(|r| r.member_path.contains(&"nested".to_string()));
    assert!(
        has_nested,
        "Failed to track deep member 'nested' across optional chain"
    );
}

#[test]
fn tracks_ts_wrappers() {
    let source = r#"
    import * as Lib from 'lib';
    
    const a = (Lib as any).prop1;
    const b = Lib!.prop2;
    const c = (Lib satisfies any).prop3;
    "#;

    let table = analyze(source);
    let lib = table
        .symbols
        .iter()
        .find(|s| s.name == "Lib")
        .expect("Lib symbol not found");

    let refs: Vec<String> = lib
        .qualified_references
        .iter()
        .flat_map(|r| r.member_path.clone())
        .collect();

    assert!(
        refs.contains(&"prop1".to_string()),
        "Failed to track through 'as'"
    );
    assert!(
        refs.contains(&"prop2".to_string()),
        "Failed to track through '!'"
    );
    assert!(
        refs.contains(&"prop3".to_string()),
        "Failed to track through 'satisfies'"
    );
}
