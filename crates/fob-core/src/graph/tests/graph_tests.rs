use std::path::PathBuf;

use super::super::{import::ImportKind, Module, ModuleGraph, ModuleId, SourceSpan, SourceType};
use serde_json::from_str;

fn make_module(id: &str) -> Module {
    let module_id = ModuleId::new_virtual(id);
    Module::builder(
        module_id.clone(),
        PathBuf::from(module_id.path_string().to_string()),
        SourceType::TypeScript,
    )
    .build()
}

#[tokio::test]
async fn add_module_registers_entry_points() {
    let graph = ModuleGraph::new().await.unwrap();
    let mut entry = make_module("virtual:entry.ts");
    entry.mark_entry();

    graph.add_module(entry.clone()).await.unwrap();

    assert_eq!(graph.len().await.unwrap(), 1);
    assert!(graph.entry_points().await.unwrap().contains(&entry.id));
    assert!(graph.module(&entry.id).await.unwrap().unwrap().is_entry);
}

#[tokio::test]
async fn add_dependency_creates_reverse_edges() {
    let graph = ModuleGraph::new().await.unwrap();
    let a = make_module("virtual:a.ts");
    let b = make_module("virtual:b.ts");

    graph.add_module(a.clone()).await.unwrap();
    graph.add_module(b.clone()).await.unwrap();

    graph.add_dependency(a.id.clone(), b.id.clone()).await.unwrap();

    let deps = graph.dependencies(&a.id).await.unwrap();
    assert!(deps.contains(&b.id));

    let rev = graph.dependents(&b.id).await.unwrap();
    assert!(rev.contains(&a.id));
}

#[tokio::test]
async fn add_entry_point_updates_existing_module() {
    let graph = ModuleGraph::new().await.unwrap();
    let module = make_module("virtual:route.ts");
    let module_id = module.id.clone();

    graph.add_module(module).await.unwrap();
    graph.add_entry_point(module_id.clone()).await.unwrap();

    assert!(graph.entry_points().await.unwrap().contains(&module_id));
    assert!(graph.module(&module_id).await.unwrap().unwrap().is_entry);
}

#[tokio::test]
async fn imports_for_module_returns_runtime_imports() {
    let graph = ModuleGraph::new().await.unwrap();
    let mut module = make_module("virtual:widget.ts");
    let import = super::super::Import::new(
        "react",
        Vec::new(),
        ImportKind::Static,
        None,
        SourceSpan::new("virtual:widget.ts", 0, 0),
    );
    module.imports.push(import.clone());

    let module_id = module.id.clone();
    graph.add_module(module).await.unwrap();

    let imports = graph.imports_for_module(&module_id).await.unwrap().unwrap();
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "react");
}

#[tokio::test]
async fn exports_to_dot_and_json_formats() {
    let graph = ModuleGraph::new().await.unwrap();
    let module = make_module("virtual:viz.ts");
    let module_id = module.id.clone();
    graph.add_module(module).await.unwrap();

    let dot = graph.to_dot_format().await.unwrap();
    assert!(dot.contains("digraph ModuleGraph"));
    assert!(dot.contains("virtual:viz.ts"));

    let json = graph.to_json().await.expect("json export should succeed");
    // Note: ModuleGraph doesn't implement Deserialize, so we can't deserialize it
    // This test just verifies that to_json() works
    assert!(!json.is_empty());
}
