//! Integration tests for export usage count tracking.
//!
//! Tests the compute_export_usage_counts() functionality across various scenarios.

use fob_graph::{
    Export, ExportKind, Import, ImportKind, ImportSpecifier, Module, ModuleGraph, ModuleId,
    SourceSpan, SourceType,
};
use fob_bundler::Result;

/// Helper to create a basic module with given exports.
fn create_module(id: &str, exports: Vec<(&str, ExportKind)>, imports: Vec<Import>) -> Module {
    let exports = exports
        .into_iter()
        .map(|(name, kind)| {
            Export::new(
                name,
                kind,
                false,
                false,
                None,
                false,
                false,
                SourceSpan::new(id, 0, 0),
            )
        })
        .collect();

    Module::builder(
        ModuleId::new(id).unwrap(),
        id.into(),
        SourceType::JavaScript,
    )
    .imports(imports)
    .exports(exports)
    .build()
}

/// Helper to create an import from a module.
fn create_import(
    source: &str,
    specifiers: Vec<ImportSpecifier>,
    resolved_to: Option<ModuleId>,
) -> Import {
    Import::new(
        source,
        specifiers,
        ImportKind::Static,
        resolved_to,
        SourceSpan::new(source, 0, 0),
    )
}

#[tokio::test]
async fn test_simple_usage_once() -> Result<()> {
    // Module A exports "foo"
    // Module B imports { foo } from A
    // Expected: foo has usage_count = 1

    let graph = ModuleGraph::new()?;

    let module_a = create_module("a.js", vec![("foo", ExportKind::Named)], vec![]);
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Named("foo".to_string())],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    // Compute usage counts
    graph.compute_export_usage_counts()?;

    // Verify
    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    assert_eq!(module_a.exports.len(), 1);
    assert_eq!(module_a.exports[0].name, "foo");
    assert_eq!(module_a.exports[0].usage_count(), Some(1));

    Ok(())
}

#[tokio::test]
async fn test_multiple_importers() -> Result<()> {
    // Module A exports "foo"
    // Module B imports { foo } from A
    // Module C imports { foo } from A
    // Expected: foo has usage_count = 2

    let graph = ModuleGraph::new()?;

    let module_a = create_module("a.js", vec![("foo", ExportKind::Named)], vec![]);
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Named("foo".to_string())],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );
    let module_c = create_module(
        "c.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Named("foo".to_string())],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_module(module_c.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;
    graph.add_dependency(module_c.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    assert_eq!(module_a.exports[0].usage_count(), Some(2));

    Ok(())
}

#[tokio::test]
async fn test_unused_export() -> Result<()> {
    // Module A exports "foo" and "bar"
    // Module B imports { foo } from A
    // Expected: foo has usage_count = 1, bar has usage_count = 0

    let graph = ModuleGraph::new()?;

    let module_a = create_module(
        "a.js",
        vec![("foo", ExportKind::Named), ("bar", ExportKind::Named)],
        vec![],
    );
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Named("foo".to_string())],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let foo = module_a.exports.iter().find(|e| e.name == "foo").unwrap();
    let bar = module_a.exports.iter().find(|e| e.name == "bar").unwrap();

    assert_eq!(foo.usage_count(), Some(1));
    assert_eq!(bar.usage_count(), Some(0));

    Ok(())
}

#[tokio::test]
async fn test_namespace_import() -> Result<()> {
    // Module A exports "foo" and "bar"
    // Module B: import * as A from "a.js"
    // Expected: both exports have usage_count = 1

    let graph = ModuleGraph::new()?;

    let module_a = create_module(
        "a.js",
        vec![("foo", ExportKind::Named), ("bar", ExportKind::Named)],
        vec![],
    );
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Namespace("A".to_string())],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let foo = module_a.exports.iter().find(|e| e.name == "foo").unwrap();
    let bar = module_a.exports.iter().find(|e| e.name == "bar").unwrap();

    // Namespace imports count as 1 usage for each export
    assert_eq!(foo.usage_count(), Some(1));
    assert_eq!(bar.usage_count(), Some(1));

    Ok(())
}

#[tokio::test]
async fn test_default_export() -> Result<()> {
    // Module A exports default
    // Module B: import A from "a.js"
    // Expected: default export has usage_count = 1

    let graph = ModuleGraph::new()?;

    let module_a = create_module("a.js", vec![("default", ExportKind::Default)], vec![]);
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![ImportSpecifier::Default],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let default_export = module_a
        .exports
        .iter()
        .find(|e| e.name == "default")
        .unwrap();

    assert_eq!(default_export.usage_count(), Some(1));

    Ok(())
}

#[tokio::test]
async fn test_multiple_imports_same_module() -> Result<()> {
    // Module A exports "foo" and "bar"
    // Module B imports { foo, bar } from A
    // Expected: foo has usage_count = 1, bar has usage_count = 1

    let graph = ModuleGraph::new()?;

    let module_a = create_module(
        "a.js",
        vec![("foo", ExportKind::Named), ("bar", ExportKind::Named)],
        vec![],
    );
    let module_b = create_module(
        "b.js",
        vec![],
        vec![create_import(
            "a.js",
            vec![
                ImportSpecifier::Named("foo".to_string()),
                ImportSpecifier::Named("bar".to_string()),
            ],
            Some(ModuleId::new("a.js").unwrap()),
        )],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let foo = module_a.exports.iter().find(|e| e.name == "foo").unwrap();
    let bar = module_a.exports.iter().find(|e| e.name == "bar").unwrap();

    assert_eq!(foo.usage_count(), Some(1));
    assert_eq!(bar.usage_count(), Some(1));

    Ok(())
}

#[tokio::test]
async fn test_same_export_multiple_imports() -> Result<()> {
    // Module A exports "foo"
    // Module B has two separate import statements both importing foo
    // Expected: foo has usage_count = 2

    let graph = ModuleGraph::new()?;

    let module_a = create_module("a.js", vec![("foo", ExportKind::Named)], vec![]);
    let module_b = create_module(
        "b.js",
        vec![],
        vec![
            create_import(
                "a.js",
                vec![ImportSpecifier::Named("foo".to_string())],
                Some(ModuleId::new("a.js").unwrap()),
            ),
            create_import(
                "a.js",
                vec![ImportSpecifier::Named("foo".to_string())],
                Some(ModuleId::new("a.js").unwrap()),
            ),
        ],
    );

    graph.add_module(module_a)?;
    graph.add_module(module_b.clone())?;
    graph.add_dependency(module_b.id.clone(), ModuleId::new("a.js").unwrap())?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let foo = module_a.exports.iter().find(|e| e.name == "foo").unwrap();

    // Two separate import statements = count of 2
    assert_eq!(foo.usage_count(), Some(2));

    Ok(())
}

#[tokio::test]
async fn test_no_imports() -> Result<()> {
    // Module A exports "foo" but no one imports it
    // Expected: foo has usage_count = 0

    let graph = ModuleGraph::new()?;

    let module_a = create_module("a.js", vec![("foo", ExportKind::Named)], vec![]);

    graph.add_module(module_a)?;

    graph.compute_export_usage_counts()?;

    let module_a = graph.module(&ModuleId::new("a.js").unwrap())?.unwrap();
    let foo = module_a.exports.iter().find(|e| e.name == "foo").unwrap();

    assert_eq!(foo.usage_count(), Some(0));

    Ok(())
}
