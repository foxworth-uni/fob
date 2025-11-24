use serde_json::{from_str, to_string};

use super::super::{Import, ImportKind, ImportSpecifier, ModuleId, SourceSpan};

#[test]
fn detects_side_effect_only_imports() {
    let import = Import::new(
        "core-js",
        Vec::new(),
        ImportKind::Static,
        None,
        dummy_span(),
    );
    assert!(import.is_side_effect_only());

    let not_side_effect = Import::new(
        "./utils",
        vec![ImportSpecifier::Named("format".into())],
        ImportKind::Static,
        None,
        dummy_span(),
    );
    assert!(!not_side_effect.is_side_effect_only());
}

#[test]
fn detects_external_imports() {
    let import = Import::new("react", Vec::new(), ImportKind::Static, None, dummy_span());
    assert!(import.is_external());

    let relative = Import::new(
        "./button",
        vec![ImportSpecifier::Default],
        ImportKind::Static,
        None,
        dummy_span(),
    );
    assert!(!relative.is_external());
}

#[test]
fn kind_helpers_work() {
    assert!(ImportKind::Static.is_runtime());
    assert!(!ImportKind::TypeOnly.is_runtime());
    assert!(ImportKind::Static.is_static());
    assert!(ImportKind::Require.is_static());
    assert!(!ImportKind::Dynamic.is_static());
}

#[test]
fn serde_roundtrip_preserves_import() {
    let module_id = ModuleId::new_virtual("virtual:dep");
    let import = Import::new(
        "virtual:dep",
        vec![ImportSpecifier::Namespace("dep".into())],
        ImportKind::Dynamic,
        Some(module_id.clone()),
        dummy_span(),
    );

    let json = to_string(&import).unwrap();
    let restored: Import = from_str(&json).unwrap();

    assert_eq!(restored.source, import.source);
    assert_eq!(restored.kind, import.kind);
    assert_eq!(restored.specifiers, import.specifiers);
    assert_eq!(restored.resolved_to, Some(module_id));
}

fn dummy_span() -> SourceSpan {
    SourceSpan::new("/tmp/module.ts", 0, 0)
}
