use std::path::PathBuf;

use super::super::{Module, ModuleGraph, ModuleId, SourceType};

fn module(id: &str) -> Module {
    let module_id = ModuleId::new_virtual(id);
    Module::builder(
        module_id.clone(),
        PathBuf::from(module_id.path_string().to_string()),
        SourceType::JavaScript,
    )
    .build()
}

#[tokio::test]
async fn depends_on_detects_transitive_edges() {
    let graph = ModuleGraph::new().await.unwrap();
    let a = module("virtual:a.js");
    let b = module("virtual:b.js");
    let c = module("virtual:c.js");

    graph.add_module(a.clone()).await.unwrap();
    graph.add_module(b.clone()).await.unwrap();
    graph.add_module(c.clone()).await.unwrap();

    graph
        .add_dependency(a.id.clone(), b.id.clone())
        .await
        .unwrap();
    graph
        .add_dependency(b.id.clone(), c.id.clone())
        .await
        .unwrap();

    assert!(graph.depends_on(&a.id, &c.id).await.unwrap());
    assert!(!graph.depends_on(&c.id, &a.id).await.unwrap());
}

#[tokio::test]
async fn transitive_dependencies_collects_unique_ids() {
    let graph = ModuleGraph::new().await.unwrap();
    let a = module("virtual:a.js");
    let b = module("virtual:b.js");
    let c = module("virtual:c.js");
    let d = module("virtual:d.js");

    graph.add_module(a.clone()).await.unwrap();
    graph.add_module(b.clone()).await.unwrap();
    graph.add_module(c.clone()).await.unwrap();
    graph.add_module(d.clone()).await.unwrap();

    graph
        .add_dependency(a.id.clone(), b.id.clone())
        .await
        .unwrap();
    graph
        .add_dependency(a.id.clone(), c.id.clone())
        .await
        .unwrap();
    graph
        .add_dependency(c.id.clone(), d.id.clone())
        .await
        .unwrap();

    let deps = graph.transitive_dependencies(&a.id).await.unwrap();

    assert_eq!(deps.len(), 3);
    assert!(deps.contains(&b.id));
    assert!(deps.contains(&c.id));
    assert!(deps.contains(&d.id));
}

#[tokio::test]
async fn depends_on_handles_cycles_without_infinite_loop() {
    let graph = ModuleGraph::new().await.unwrap();
    let a = module("virtual:cycle_a.js");
    let b = module("virtual:cycle_b.js");

    graph.add_module(a.clone()).await.unwrap();
    graph.add_module(b.clone()).await.unwrap();

    graph
        .add_dependency(a.id.clone(), b.id.clone())
        .await
        .unwrap();
    graph
        .add_dependency(b.id.clone(), a.id.clone())
        .await
        .unwrap();

    assert!(graph.depends_on(&a.id, &b.id).await.unwrap());
    assert!(graph.depends_on(&b.id, &a.id).await.unwrap());
}
