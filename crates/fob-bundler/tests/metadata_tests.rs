//! Integration tests for bundle metadata extraction.
//!
//! These tests verify that metadata is correctly extracted from module graphs,
//! including exports, imports, and aggregate statistics.

use fob_bundler::graph::{
    Export, ExportKind, Import, ImportKind, ImportSpecifier, Module, ModuleGraph, ModuleId,
    SourceSpan, SourceType,
};
use fob_bundler::output::metadata::BundleMetadata;

/// Helper to create a test module graph with various scenarios.
async fn create_test_graph() -> ModuleGraph {
    let graph = ModuleGraph::new().await.unwrap();

    // Entry module with default and named exports
    let entry_id = ModuleId::new_virtual("entry.js");
    let entry = Module::builder(entry_id.clone(), "entry.js".into(), SourceType::JavaScript)
        .exports(vec![
            Export::new(
                "Component",
                ExportKind::Named,
                true, // used
                false,
                None,
                false,
                false,
                SourceSpan::new("entry.js", 0, 0),
            ),
            Export::new(
                "Helper",
                ExportKind::Named,
                false, // unused
                false,
                None,
                false,
                false,
                SourceSpan::new("entry.js", 0, 0),
            ),
            Export::new(
                "default",
                ExportKind::Default,
                true,
                false,
                None,
                false,
                false,
                SourceSpan::new("entry.js", 0, 0),
            ),
        ])
        .imports(vec![
            Import::new(
                "react",
                vec![
                    ImportSpecifier::Named("useState".into()),
                    ImportSpecifier::Named("useEffect".into()),
                ],
                ImportKind::Static,
                None,
                SourceSpan::new("entry.js", 0, 0),
            ),
            Import::new(
                "lodash",
                vec![ImportSpecifier::Named("debounce".into())],
                ImportKind::Static,
                None,
                SourceSpan::new("entry.js", 0, 0),
            ),
        ])
        .entry(true)
        .original_size(1024)
        .build();

    graph.add_module(entry).await.unwrap();

    // Utility module with re-exports
    let utils_id = ModuleId::new_virtual("utils.js");
    let utils = Module::builder(utils_id.clone(), "utils.js".into(), SourceType::JavaScript)
        .exports(vec![
            Export::new(
                "formatDate",
                ExportKind::Named,
                true,
                false,
                None,
                false,
                false,
                SourceSpan::new("utils.js", 0, 0),
            ),
            Export::new(
                "parseDate",
                ExportKind::Named,
                true,
                false,
                Some("date-fns".to_string()),
                false,
                false,
                SourceSpan::new("utils.js", 0, 0),
            ),
        ])
        .imports(vec![Import::new(
            "date-fns",
            vec![ImportSpecifier::Named("format".into())],
            ImportKind::Static,
            None,
            SourceSpan::new("utils.js", 0, 0),
        )])
        .original_size(512)
        .build();

    graph.add_module(utils).await.unwrap();

    // Type-only module (TypeScript)
    let types_id = ModuleId::new_virtual("types.ts");
    let types = Module::builder(types_id.clone(), "types.ts".into(), SourceType::TypeScript)
        .exports(vec![Export::new(
            "User",
            ExportKind::Named,
            true,
            true, // type-only
            None,
            false,
            false,
            SourceSpan::new("types.ts", 0, 0),
        )])
        .imports(vec![Import::new(
            "zod",
            vec![ImportSpecifier::Named("z".into())],
            ImportKind::TypeOnly,
            None,
            SourceSpan::new("types.ts", 0, 0),
        )])
        .original_size(256)
        .build();

    graph.add_module(types).await.unwrap();

    graph
}

#[tokio::test]
async fn test_metadata_extracts_all_exports() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let exports = metadata.exports();

    // Should have exports from all modules
    assert_eq!(exports.len(), 6); // 3 from entry + 2 from utils + 1 from types

    // Check for specific exports
    assert!(exports.iter().any(|e| e.name == "Component"));
    assert!(exports.iter().any(|e| e.name == "Helper"));
    assert!(exports.iter().any(|e| e.name == "formatDate"));
    assert!(exports.iter().any(|e| e.name == "parseDate"));
    assert!(exports.iter().any(|e| e.name == "User"));
}

#[tokio::test]
async fn test_metadata_identifies_default_export() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    assert!(metadata.has_default_export());

    let default_exports = metadata.default_exports();
    assert_eq!(default_exports.len(), 1);
    assert_eq!(default_exports[0].name, "default");
    assert!(default_exports[0].is_default);
}

#[tokio::test]
async fn test_metadata_separates_named_and_default_exports() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let named = metadata.named_exports();
    let default = metadata.default_exports();

    // Should have 5 named exports and 1 default
    assert_eq!(named.len(), 5);
    assert_eq!(default.len(), 1);

    // Named exports shouldn't include default
    assert!(!named.iter().any(|e| e.is_default));

    // Default exports should all be defaults
    assert!(default.iter().all(|e| e.is_default));
}

#[tokio::test]
async fn test_metadata_tracks_export_usage() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let exports = metadata.exports();

    // Component should be marked as used
    let component = exports.iter().find(|e| e.name == "Component").unwrap();
    assert!(component.is_used);

    // Helper should be marked as unused
    let helper = exports.iter().find(|e| e.name == "Helper").unwrap();
    assert!(!helper.is_used);
}

#[tokio::test]
async fn test_metadata_identifies_unused_exports() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let unused = metadata.unused_exports();

    // Should have at least one unused export (Helper)
    assert!(!unused.is_empty());
    assert!(unused.iter().any(|e| e.name == "Helper"));

    // All unused exports should be marked as not used
    assert!(unused.iter().all(|e| !e.is_used));
}

#[tokio::test]
async fn test_metadata_extracts_external_imports() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let imports = metadata.imports();

    // Should have external imports from npm packages
    assert!(!imports.is_empty());

    // Check for specific external dependencies
    let specifiers: Vec<_> = imports.iter().map(|i| i.specifier.as_str()).collect();
    assert!(specifiers.contains(&"react"));
    assert!(specifiers.contains(&"lodash"));
    assert!(specifiers.contains(&"date-fns"));
}

#[tokio::test]
async fn test_metadata_groups_imports_by_specifier() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let imports = metadata.imports();

    // date-fns is imported by both entry.js and utils.js
    let date_fns = imports.iter().find(|i| i.specifier == "date-fns").unwrap();

    // Should track all modules that import it
    assert!(!date_fns.imported_by.is_empty());
}

#[tokio::test]
async fn test_metadata_identifies_type_only_imports() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 2048, 1).await.unwrap();

    let imports = metadata.imports();

    // zod import should be marked as type-only
    let zod = imports.iter().find(|i| i.specifier == "zod");
    assert!(zod.is_some());
    assert!(zod.unwrap().is_type_only);

    // react import should be runtime
    let react = imports.iter().find(|i| i.specifier == "react").unwrap();
    assert!(!react.is_type_only);
}

#[tokio::test]
async fn test_metadata_calculates_total_size() {
    let graph = create_test_graph().await;
    let total_size = 2048;
    let metadata = BundleMetadata::from_graph(&graph, total_size, 1)
        .await
        .unwrap();

    assert_eq!(metadata.total_size(), total_size);
}

#[tokio::test]
async fn test_metadata_counts_assets() {
    let graph = create_test_graph().await;
    let asset_count = 3;
    let metadata = BundleMetadata::from_graph(&graph, 1024, asset_count)
        .await
        .unwrap();

    assert_eq!(metadata.asset_count(), asset_count);
}

#[tokio::test]
async fn test_metadata_counts_modules() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 1024, 1).await.unwrap();

    // Should count all modules in the graph
    assert_eq!(metadata.module_count(), graph.len().await.unwrap());
    assert_eq!(metadata.module_count(), 3); // entry, utils, types
}

#[tokio::test]
async fn test_has_export_check() {
    let graph = create_test_graph().await;
    let metadata = BundleMetadata::from_graph(&graph, 1024, 1).await.unwrap();

    // Should find existing exports
    assert!(metadata.has_export("Component"));
    assert!(metadata.has_export("Helper"));
    assert!(metadata.has_export("formatDate"));
    assert!(metadata.has_export("default"));

    // Should not find non-existent exports
    assert!(!metadata.has_export("NonExistent"));
    assert!(!metadata.has_export("FakeExport"));
}

#[tokio::test]
async fn test_metadata_with_empty_graph() {
    let graph = ModuleGraph::new().await.unwrap();
    let metadata = BundleMetadata::from_graph(&graph, 0, 0).await.unwrap();

    assert_eq!(metadata.exports().len(), 0);
    assert_eq!(metadata.imports().len(), 0);
    assert_eq!(metadata.total_size(), 0);
    assert_eq!(metadata.asset_count(), 0);
    assert_eq!(metadata.module_count(), 0);
    assert!(!metadata.has_default_export());
}

#[tokio::test]
async fn test_metadata_with_single_module() {
    let graph = ModuleGraph::new().await.unwrap();

    let module_id = ModuleId::new_virtual("index.js");
    let module = Module::builder(module_id, "index.js".into(), SourceType::JavaScript)
        .exports(vec![Export::new(
            "default",
            ExportKind::Default,
            true,
            false,
            None,
            false,
            false,
            SourceSpan::new("index.js", 0, 0),
        )])
        .entry(true)
        .original_size(100)
        .build();

    graph.add_module(module).await.unwrap();

    let metadata = BundleMetadata::from_graph(&graph, 100, 1).await.unwrap();

    assert_eq!(metadata.exports().len(), 1);
    assert!(metadata.has_default_export());
    assert_eq!(metadata.module_count(), 1);
    assert_eq!(metadata.imports().len(), 0);
}

#[tokio::test]
async fn test_metadata_excludes_internal_imports() {
    let graph = ModuleGraph::new().await.unwrap();

    let module_a_id = ModuleId::new_virtual("a.js");
    let module_b_id = ModuleId::new_virtual("b.js");

    // Module A imports from Module B (internal) and React (external)
    let module_a = Module::builder(module_a_id.clone(), "a.js".into(), SourceType::JavaScript)
        .imports(vec![
            Import::new(
                "./b.js", // Internal import (relative path)
                vec![ImportSpecifier::Named("helper".into())],
                ImportKind::Static,
                Some(module_b_id.clone()),
                SourceSpan::new("a.js", 0, 0),
            ),
            Import::new(
                "react", // External import
                vec![ImportSpecifier::Named("useState".into())],
                ImportKind::Static,
                None,
                SourceSpan::new("a.js", 0, 0),
            ),
        ])
        .entry(true)
        .build();

    let module_b = Module::builder(module_b_id, "b.js".into(), SourceType::JavaScript)
        .exports(vec![Export::new(
            "helper",
            ExportKind::Named,
            true,
            false,
            None,
            false,
            false,
            SourceSpan::new("b.js", 0, 0),
        )])
        .build();

    graph.add_module(module_a).await.unwrap();
    graph.add_module(module_b).await.unwrap();

    let metadata = BundleMetadata::from_graph(&graph, 200, 1).await.unwrap();

    // Should only include external import (react), not internal (./b.js)
    assert_eq!(metadata.imports().len(), 1);
    assert_eq!(metadata.imports()[0].specifier, "react");
}

#[tokio::test]
async fn test_metadata_handles_re_exports() {
    let graph = ModuleGraph::new().await.unwrap();

    let module_id = ModuleId::new_virtual("reexports.js");
    let module = Module::builder(module_id, "reexports.js".into(), SourceType::JavaScript)
        .exports(vec![Export::new(
            "Button",
            ExportKind::ReExport,
            true,
            false,
            Some("./components/button".to_string()),
            false,
            false,
            SourceSpan::new("reexports.js", 0, 0),
        )])
        .build();

    graph.add_module(module).await.unwrap();

    let metadata = BundleMetadata::from_graph(&graph, 100, 1).await.unwrap();

    assert_eq!(metadata.exports().len(), 1);
    let export = &metadata.exports()[0];
    assert_eq!(export.name, "Button");
    assert_eq!(export.source_module, "reexports.js");
}

#[tokio::test]
async fn test_metadata_multiple_default_exports() {
    let graph = ModuleGraph::new().await.unwrap();

    // Two different modules with default exports
    let module1_id = ModuleId::new_virtual("module1.js");
    let module1 = Module::builder(module1_id, "module1.js".into(), SourceType::JavaScript)
        .exports(vec![Export::new(
            "default",
            ExportKind::Default,
            true,
            false,
            None,
            false,
            false,
            SourceSpan::new("module1.js", 0, 0),
        )])
        .build();

    let module2_id = ModuleId::new_virtual("module2.js");
    let module2 = Module::builder(module2_id, "module2.js".into(), SourceType::JavaScript)
        .exports(vec![Export::new(
            "default",
            ExportKind::Default,
            true,
            false,
            None,
            false,
            false,
            SourceSpan::new("module2.js", 0, 0),
        )])
        .build();

    graph.add_module(module1).await.unwrap();
    graph.add_module(module2).await.unwrap();

    let metadata = BundleMetadata::from_graph(&graph, 200, 1).await.unwrap();

    // Should detect that defaults exist
    assert!(metadata.has_default_export());

    // Should find both default exports
    let defaults = metadata.default_exports();
    assert_eq!(defaults.len(), 2);
}
