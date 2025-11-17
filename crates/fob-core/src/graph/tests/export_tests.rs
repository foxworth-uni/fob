use serde_json::{from_str, to_string};

use super::super::{Export, ExportKind, SourceSpan};

#[test]
fn framework_mark_sets_flags() {
    let span = SourceSpan::new("/tmp/module.ts", 0, 10);
    let mut export = Export::new(
        "useCounter",
        ExportKind::Named,
        false,
        false,
        None,
        false,
        false,
        span,
    );

    export.mark_framework_used();

    assert!(export.is_framework_used);
    assert!(export.is_used);
}

#[test]
fn identifies_default_and_re_exports() {
    let default_export = Export::new(
        "default",
        ExportKind::Default,
        false,
        false,
        None,
        false,
        false,
        SourceSpan::new("/tmp/module.ts", 0, 5),
    );
    assert!(default_export.is_default());

    let re_export = Export::new(
        "value",
        ExportKind::ReExport,
        true,
        false,
        Some("./dep".into()),
        false,
        false,
        SourceSpan::new("/tmp/module.ts", 10, 20),
    );
    assert!(re_export.is_re_export());
    assert_eq!(re_export.re_exported_from.as_deref(), Some("./dep"));
}

#[test]
fn serde_roundtrip_preserves_export() {
    let export = Export::new(
        "foo",
        ExportKind::Named,
        true,
        false,
        None,
        false,
        false,
        SourceSpan::new("/tmp/module.ts", 0, 3),
    );

    let json = to_string(&export).unwrap();
    let restored: Export = from_str(&json).unwrap();

    assert_eq!(restored.name, export.name);
    assert_eq!(restored.kind, export.kind);
    assert_eq!(restored.is_used, export.is_used);
    assert_eq!(restored.span.start, export.span.start);
}
