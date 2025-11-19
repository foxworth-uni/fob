use std::path::PathBuf;

use super::super::{
    export::ExportKind,
    import::{ImportKind, ImportSpecifier},
    Export, ExternalDependency, GraphStatistics, Import, Module, ModuleGraph, ModuleId, SourceSpan,
    SourceType,
};

fn module_with_exports(id: &str, export_names: &[&str]) -> Module {
    let module_id = ModuleId::new_virtual(id);
    let exports = export_names
        .iter()
        .map(|name| {
            Export::new(
                name.to_string(),
                ExportKind::Named,
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
        module_id.clone(),
        PathBuf::from(module_id.path_string().to_string()),
        SourceType::TypeScript,
    )
    .exports(exports)
    .build()
}

fn import_named(from: &str, name: &str) -> Import {
    Import::new(
        from,
        vec![ImportSpecifier::Named(name.to_string())],
        ImportKind::Static,
        Some(ModuleId::new_virtual(from)),
        SourceSpan::new(from, 0, 0),
    )
}

fn import_star_reexport(from: &str) -> Import {
    Import::new(
        from,
        vec![ImportSpecifier::Namespace("*".into())],
        ImportKind::ReExport,
        Some(ModuleId::new_virtual(from)),
        SourceSpan::new(from, 0, 0),
    )
}

fn import_namespace(from: &str) -> Import {
    Import::new(
        from,
        vec![ImportSpecifier::Namespace("utils".into())],
        ImportKind::Static,
        Some(ModuleId::new_virtual(from)),
        SourceSpan::new(from, 0, 0),
    )
}

#[tokio::test]
async fn unused_exports_filters_consumed_symbols() {
    let graph = ModuleGraph::new().await.unwrap();

    let utils = module_with_exports("virtual:utils.ts", &["format", "slug"]);
    let mut ui = module_with_exports("virtual:ui.tsx", &["Button"]);
    ui.mark_entry();

    ui.imports.push(import_named("virtual:utils.ts", "format"));

    graph.add_module(utils.clone()).await.unwrap();
    graph.add_module(ui.clone()).await.unwrap();

    graph.add_dependency(ui.id.clone(), utils.id.clone()).await.unwrap();

    let unused = graph.unused_exports().await.unwrap();
    assert_eq!(unused.len(), 1);
    assert_eq!(unused[0].export.name, "slug");
    assert_eq!(unused[0].module_id, utils.id);
}

#[tokio::test]
async fn unreachable_modules_ignores_entry_and_side_effects() {
    let graph = ModuleGraph::new().await.unwrap();

    let mut entry = module_with_exports("virtual:entry.ts", &[]);
    entry.mark_entry();

    let mut side_effect = module_with_exports("virtual:polyfill.ts", &[]);
    side_effect.set_side_effects(true);

    let orphan = module_with_exports("virtual:unused.ts", &[]);

    graph.add_module(entry).await.unwrap();
    graph.add_module(side_effect).await.unwrap();
    graph.add_module(orphan.clone()).await.unwrap();

    let unreachable = graph.unreachable_modules().await.unwrap();
    assert_eq!(unreachable.len(), 1);
    assert_eq!(unreachable[0].id, orphan.id);
}

#[tokio::test]
async fn external_dependencies_aggregates_importers() {
    let graph = ModuleGraph::new().await.unwrap();

    let mut entry = module_with_exports("virtual:entry.tsx", &[]);
    entry.imports.push(Import::new(
        "react",
        Vec::new(),
        ImportKind::Static,
        None,
        SourceSpan::new("virtual:entry.tsx", 0, 0),
    ));

    let mut another = module_with_exports("virtual:another.tsx", &[]);
    another.imports.push(Import::new(
        "react",
        Vec::new(),
        ImportKind::Static,
        None,
        SourceSpan::new("virtual:another.tsx", 0, 0),
    ));

    graph.add_module(entry.clone()).await.unwrap();
    graph.add_module(another.clone()).await.unwrap();

    let externals = graph.external_dependencies().await.unwrap();
    assert_eq!(externals.len(), 1);
    assert_eq!(externals[0].specifier, "react");
    assert_eq!(externals[0].importers.len(), 2);
}

#[tokio::test]
async fn statistics_reflects_graph_state() {
    let graph = ModuleGraph::new().await.unwrap();

    let mut entry = module_with_exports("virtual:entry.ts", &["run"]);
    entry.mark_entry();

    let mut util = module_with_exports("virtual:util.ts", &["helper"]);
    util.imports.push(import_named("virtual:entry.ts", "run"));
    util.imports.push(Import::new(
        "legacy-lib",
        Vec::new(),
        ImportKind::Static,
        None,
        SourceSpan::new("virtual:util.ts", 0, 0),
    ));
    graph.add_module(entry.clone()).await.unwrap();
    graph.add_module(util.clone()).await.unwrap();
    graph.add_dependency(util.id.clone(), entry.id.clone()).await.unwrap();

    graph.add_external_dependency(ExternalDependency {
        specifier: "legacy-lib".into(),
        importers: vec![entry.id.clone()],
    }).await.unwrap();

    let unused_count = graph.unused_exports().await.unwrap().len();
    let stats = graph.statistics().await.unwrap();
    assert_eq!(
        stats,
        GraphStatistics::new(2, 1, 1, 0, unused_count, 1)
    );
}

#[tokio::test]
async fn star_reexport_doesnt_mark_all_exports_used() {
    // Test case: export * from './validators' doesn't mark all exports as used
    // validators.ts: export const validateEmail = () => {}
    //                export const validateZipCode = () => {}
    // helpers.ts: export * from './validators' (star re-export)
    // demo.tsx: import { validateEmail } from './helpers'
    // Expected: validateZipCode should be unused
    
    let graph = ModuleGraph::new().await.unwrap();
    
    let validators_id = ModuleId::new_virtual("validators.ts");
    let validators = Module::builder(
        validators_id.clone(),
        PathBuf::from("validators.ts"),
        SourceType::TypeScript,
    )
    .exports(vec![
        Export::new(
            "validateEmail",
            ExportKind::Named,
            false,
            false,
            None,
            false,
            false,
            SourceSpan::new("validators.ts", 0, 0),
        ),
        Export::new(
            "validateZipCode",
            ExportKind::Named,
            false,
            false,
            None,
            false,
            false,
            SourceSpan::new("validators.ts", 0, 0),
        ),
    ])
    .build();
    
    let helpers_id = ModuleId::new_virtual("helpers.ts");
    let mut helpers = Module::builder(
        helpers_id.clone(),
        PathBuf::from("helpers.ts"),
        SourceType::TypeScript,
    )
    .exports(vec![
        Export::new(
            "validateEmail",
            ExportKind::ReExport,
            false,
            false,
            Some("validators.ts".to_string()),
            false,
            false,
            SourceSpan::new("helpers.ts", 0, 0),
        ),
        Export::new(
            "validateZipCode",
            ExportKind::ReExport,
            false,
            false,
            Some("validators.ts".to_string()),
            false,
            false,
            SourceSpan::new("helpers.ts", 0, 0),
        ),
    ])
    .build();
    helpers.imports.push(import_star_reexport("validators.ts"));
    
    let mut demo = module_with_exports("demo.tsx", &[]);
    demo.mark_entry();
    demo.imports.push(import_named("helpers.ts", "validateEmail"));
    
    graph.add_module(validators.clone()).await.unwrap();
    graph.add_module(helpers.clone()).await.unwrap();
    graph.add_module(demo.clone()).await.unwrap();
    
    graph.add_dependency(helpers_id.clone(), validators_id.clone()).await.unwrap();
    graph.add_dependency(demo.id.clone(), helpers_id.clone()).await.unwrap();
    
    let unused = graph.unused_exports().await.unwrap();
    
    // validateZipCode should be unused (not marked as used by star re-export)
    let unused_zipcode = unused.iter().find(|u| {
        u.module_id == validators_id && u.export.name == "validateZipCode"
    });
    assert!(unused_zipcode.is_some(), "validateZipCode should be unused");
    
    // validateEmail should NOT be unused (it's imported)
    let unused_email = unused.iter().find(|u| {
        u.module_id == validators_id && u.export.name == "validateEmail"
    });
    assert!(unused_email.is_none(), "validateEmail should be used");
}

#[tokio::test]
async fn namespace_import_marks_all_exports_used() {
    // Test case: import * as utils from './utils' marks ALL exports as used
    // utils.ts: export const foo = 1;
    //           export const bar = 2;
    // app.ts: import * as utils from './utils';
    // Expected: Both foo and bar should be marked as used
    
    let graph = ModuleGraph::new().await.unwrap();
    
    let utils_id = ModuleId::new_virtual("utils.ts");
    let utils = Module::builder(
        utils_id.clone(),
        PathBuf::from("utils.ts"),
        SourceType::TypeScript,
    )
    .exports(vec![
        Export::new(
            "foo",
            ExportKind::Named,
            false,
            false,
            None,
            false,
            false,
            SourceSpan::new("utils.ts", 0, 0),
        ),
        Export::new(
            "bar",
            ExportKind::Named,
            false,
            false,
            None,
            false,
            false,
            SourceSpan::new("utils.ts", 0, 0),
        ),
    ])
    .build();
    
    let mut app = module_with_exports("app.ts", &[]);
    app.mark_entry();
    app.imports.push(import_namespace("utils.ts"));
    
    graph.add_module(utils.clone()).await.unwrap();
    graph.add_module(app.clone()).await.unwrap();
    
    graph.add_dependency(app.id.clone(), utils_id.clone()).await.unwrap();
    
    let unused = graph.unused_exports().await.unwrap();
    
    // Both foo and bar should be marked as used by namespace import
    let unused_from_utils: Vec<_> = unused
        .iter()
        .filter(|u| u.module_id == utils_id)
        .collect();
    assert_eq!(unused_from_utils.len(), 0, "All exports should be used by namespace import");
}
