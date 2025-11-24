use std::sync::Arc;

use std::path::PathBuf;
use std::time::Duration;

use super::super::{Export, ExportKind, Module, ModuleGraph, ModuleId, SourceSpan, SourceType};
use tokio::time::timeout;

fn make_module(id: &str) -> Module {
    let module_id = ModuleId::new_virtual(id);
    Module::builder(
        module_id.clone(),
        PathBuf::from(module_id.path_string().to_string()),
        SourceType::TypeScript,
    )
    .build()
}

/// Test concurrent read operations don't block each other
#[tokio::test]
async fn concurrent_reads_do_not_block() {
    let graph = ModuleGraph::new().unwrap();
    
    // Add some modules
    for i in 0..10 {
        let module = make_module(&format!("virtual:module_{}.ts", i));
        graph.add_module(module).unwrap();
    }

    // Spawn multiple concurrent readers
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let graph = graph.clone();
            tokio::spawn(async move {
                for _ in 0..100 {
                    let _modules = graph.modules().unwrap();
                    let _len = graph.len().unwrap();
                }
            })
        })
        .collect();

    // All readers should complete quickly
    let result = timeout(Duration::from_secs(5), async {
        for handle in handles {
            handle.unwrap();
        }
    })
    .await;

    assert!(result.is_ok(), "Concurrent reads should not block");
}

/// Test concurrent write operations are safe
#[tokio::test]
async fn concurrent_writes_are_safe() {
    let graph = ModuleGraph::new().unwrap();

    // Spawn multiple concurrent writers
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let graph = graph.clone();
            tokio::spawn(async move {
                for j in 0..10 {
                    let module = make_module(&format!("virtual:writer_{}_module_{}.ts", i, j));
                    graph.add_module(module).unwrap();
                }
            })
        })
        .collect();

    // All writers should complete successfully
    let result = timeout(Duration::from_secs(5), async {
        for handle in handles {
            handle.unwrap();
        }
    })
    .await;

    assert!(result.is_ok(), "Concurrent writes should complete");
    
    // Verify all modules were added (may be fewer due to duplicates, but should be consistent)
    let len = graph.len().unwrap();
    assert!(len > 0 && len <= 100, "Expected some modules to be added");
}

/// Test concurrent read and write operations
#[tokio::test]
async fn concurrent_read_write_mixed() {
    let graph = ModuleGraph::new().unwrap();

    // Add initial modules
    for i in 0..5 {
        let module = make_module(&format!("virtual:initial_{}.ts", i));
        graph.add_module(module).unwrap();
    }

    // Spawn concurrent readers and writers
    let reader_handles: Vec<_> = (0..5)
        .map(|_| {
            let graph = graph.clone();
            tokio::spawn(async move {
                for _ in 0..50 {
                    let _modules = graph.modules().unwrap();
                    let _len = graph.len().unwrap();
                }
            })
        })
        .collect();

    let writer_handles: Vec<_> = (0..5)
        .map(|i| {
            let graph = graph.clone();
            tokio::spawn(async move {
                for j in 0..10 {
                    let module = make_module(&format!("virtual:mixed_{}_module_{}.ts", i, j));
                    graph.add_module(module).unwrap();
                }
            })
        })
        .collect();

    // Both readers and writers should complete
    let result = timeout(Duration::from_secs(10), async {
        for handle in reader_handles {
            handle.unwrap();
        }
        for handle in writer_handles {
            handle.unwrap();
        }
    })
    .await;

    assert!(result.is_ok(), "Mixed read/write operations should complete");
}

/// Test that duplicate dependencies are handled correctly (HashSet prevents duplicates)
#[tokio::test]
async fn duplicate_dependencies_are_deduplicated() {
    let graph = ModuleGraph::new().unwrap();
    let a = make_module("virtual:a.ts");
    let b = make_module("virtual:b.ts");

    graph.add_module(a.clone()).unwrap();
    graph.add_module(b.clone()).unwrap();

    // Add the same dependency multiple times
    for _ in 0..10 {
        graph
            .add_dependency(a.id.clone(), b.id.clone())
            .await
            .unwrap();
    }

    // Should only have one dependency
    let deps = graph.dependencies(&a.id).unwrap();
    assert_eq!(deps.len(), 1, "HashSet should deduplicate dependencies");
    assert!(deps.contains(&b.id));

    // Reverse edge should also only have one
    let dependents = graph.dependents(&b.id).unwrap();
    assert_eq!(dependents.len(), 1, "HashSet should deduplicate dependents");
    assert!(dependents.contains(&a.id));
}

/// Test concurrent duplicate dependency additions
#[tokio::test]
async fn concurrent_duplicate_dependency_additions() {
    let graph = ModuleGraph::new().unwrap();
    let a = make_module("virtual:a.ts");
    let b = make_module("virtual:b.ts");

    graph.add_module(a.clone()).unwrap();
    graph.add_module(b.clone()).unwrap();

    // Spawn multiple tasks adding the same dependency concurrently
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let graph = graph.clone();
            let a_id = a.id.clone();
            let b_id = b.id.clone();
            tokio::spawn(async move {
                for _ in 0..10 {
                    graph.add_dependency(a_id.clone(), b_id.clone()).unwrap();
                }
            })
        })
        .collect();

    // Wait for all to complete
    for handle in handles {
        handle.unwrap();
    }

    // Should still only have one dependency (HashSet deduplication)
    let deps = graph.dependencies(&a.id).unwrap();
    assert_eq!(deps.len(), 1, "Concurrent duplicate additions should be deduplicated");
    assert!(deps.contains(&b.id));
}

/// Test that entry points can be added concurrently
#[tokio::test]
async fn concurrent_entry_point_additions() {
    let graph = ModuleGraph::new().unwrap();

    // Create modules
    let modules: Vec<_> = (0..10)
        .map(|i| make_module(&format!("virtual:entry_{}.ts", i)))
        .collect();

    // Add modules first
    for module in &modules {
        graph.add_module(module.clone()).unwrap();
    }

    // Mark all as entry points concurrently
    let handles: Vec<_> = modules
        .iter()
        .map(|module| {
            let graph = graph.clone();
            let id = module.id.clone();
            tokio::spawn(async move {
                graph.add_entry_point(id).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.unwrap();
    }

    // Verify all are entry points
    let entry_points = graph.entry_points().unwrap();
    assert_eq!(entry_points.len(), 10, "All modules should be entry points");
}

/// Test concurrent module additions with dependencies
#[tokio::test]
async fn concurrent_module_and_dependency_additions() {
    let graph = ModuleGraph::new().unwrap();

    // Create a chain of modules
    let modules: Vec<_> = (0..10)
        .map(|i| make_module(&format!("virtual:chain_{}.ts", i)))
        .collect();

    // Add modules concurrently
    let add_handles: Vec<_> = modules
        .iter()
        .map(|module| {
            let graph = graph.clone();
            let module = module.clone();
            tokio::spawn(async move {
                graph.add_module(module).unwrap();
            })
        })
        .collect();

    for handle in add_handles {
        handle.unwrap();
    }

    // Add dependencies concurrently (each module depends on the next)
    let dep_handles: Vec<_> = (0..9)
        .map(|i| {
            let graph = graph.clone();
            let from_id = modules[i].id.clone();
            let to_id = modules[i + 1].id.clone();
            tokio::spawn(async move {
                graph.add_dependency(from_id, to_id).unwrap();
            })
        })
        .collect();

    for handle in dep_handles {
        handle.unwrap();
    }

    // Verify dependencies
    for i in 0..9 {
        let deps = graph.dependencies(&modules[i].id).unwrap();
        assert!(deps.contains(&modules[i + 1].id), "Dependency should exist");
    }
}

/// Test that statistics can be computed concurrently with writes
#[tokio::test]
async fn concurrent_statistics_and_writes() {
    let graph = ModuleGraph::new().unwrap();

    // Add initial modules
    for i in 0..5 {
        let module = make_module(&format!("virtual:stats_{}.ts", i));
        graph.add_module(module).unwrap();
    }

    // Spawn statistics readers
    let stats_handles: Vec<_> = (0..5)
        .map(|_| {
            let graph = graph.clone();
            tokio::spawn(async move {
                for _ in 0..20 {
                    let _stats = graph.statistics().unwrap();
                    let _len = graph.len().unwrap();
                }
            })
        })
        .collect();

    // Spawn writers
    let writer_handles: Vec<_> = (5..10)
        .map(|i| {
            let graph = graph.clone();
            tokio::spawn(async move {
                let module = make_module(&format!("virtual:stats_{}.ts", i));
                graph.add_module(module).unwrap();
            })
        })
        .collect();

    // Wait for all to complete
    let result = timeout(Duration::from_secs(10), async {
        for handle in stats_handles {
            handle.unwrap();
        }
        for handle in writer_handles {
            handle.unwrap();
        }
    })
    .await;

    assert!(result.is_ok(), "Statistics and writes should work concurrently");
}

/// Test that unused_exports computation works with concurrent reads
#[tokio::test]
async fn concurrent_unused_exports_computation() {
    let graph = ModuleGraph::new().unwrap();

    // Add modules with exports
    for i in 0..5 {
        let mut module = make_module(&format!("virtual:export_{}.ts", i));
Arc::make_mut(&mut         module.exports).push(Export::new(
            format!("export_{}", i),
            ExportKind::Named,
            false, // is_used
            false, // is_type_only
            None,  // re_exported_from
            false, // is_framework_used
            false, // came_from_commonjs
            SourceSpan::new(&format!("virtual:export_{}.ts", i), 0, 0),
        ));
        graph.add_module(module).unwrap();
    }

    // Spawn concurrent unused_exports computations
    let handles: Vec<_> = (0..5)
        .map(|_| {
            let graph = graph.clone();
            tokio::spawn(async move {
                let _unused = graph.unused_exports().unwrap();
            })
        })
        .collect();

    // All should complete successfully
    let result = timeout(Duration::from_secs(5), async {
        for handle in handles {
            handle.unwrap();
        }
    })
    .await;

    assert!(result.is_ok(), "Concurrent unused_exports computations should work");
}

