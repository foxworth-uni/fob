//! Thread-safe concurrent access example.
//!
//! This example demonstrates:
//! - Sharing a ModuleGraph across multiple threads
//! - Concurrent read operations
//! - Thread-safe querying of dependencies and statistics
//! - Using Arc for efficient shared ownership

use fob_graph::{Module, ModuleGraph, ModuleId, SourceType};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a graph and populate it with modules
    let graph = ModuleGraph::new()?;

    // Create multiple modules
    let modules = vec![
        ("src/index.ts", true),
        ("src/utils.ts", false),
        ("src/api.ts", false),
        ("src/components.ts", false),
    ];

    for (path, is_entry) in modules {
        let id = ModuleId::new(path)?;
        let module = Module::builder(id.clone(), PathBuf::from(path), SourceType::TypeScript)
            .entry(is_entry)
            .build();
        graph.add_module(module)?;
    }

    // Add some dependencies
    let index_id = ModuleId::new("src/index.ts")?;
    let utils_id = ModuleId::new("src/utils.ts")?;
    let api_id = ModuleId::new("src/api.ts")?;
    let components_id = ModuleId::new("src/components.ts")?;

    graph.add_dependency(index_id.clone(), utils_id.clone())?;
    graph.add_dependency(index_id.clone(), api_id.clone())?;
    graph.add_dependency(api_id.clone(), utils_id.clone())?;
    graph.add_dependency(components_id.clone(), utils_id.clone())?;

    // Clone the graph (Arc-based, so this is cheap)
    let graph = Arc::new(graph);

    // Spawn multiple threads that read from the graph concurrently
    let mut handles = vec![];

    // Thread 1: Query dependencies
    let graph1 = Arc::clone(&graph);
    let index_id1 = index_id.clone();
    handles.push(thread::spawn(move || {
        let deps = graph1.dependencies(&index_id1).unwrap();
        println!("Thread 1: index.ts has {} dependencies", deps.len());
        deps.len()
    }));

    // Thread 2: Query dependents
    let graph2 = Arc::clone(&graph);
    let utils_id2 = utils_id.clone();
    handles.push(thread::spawn(move || {
        let dependents = graph2.dependents(&utils_id2).unwrap();
        println!("Thread 2: utils.ts has {} dependents", dependents.len());
        dependents.len()
    }));

    // Thread 3: Get statistics
    let graph3 = Arc::clone(&graph);
    handles.push(thread::spawn(move || {
        let stats = graph3.statistics().unwrap();
        println!(
            "Thread 3: Graph has {} modules, {} entry points",
            stats.module_count, stats.entry_point_count
        );
        stats.module_count
    }));

    // Thread 4: Query entry points
    let graph4 = Arc::clone(&graph);
    handles.push(thread::spawn(move || {
        let entries = graph4.entry_points().unwrap();
        println!("Thread 4: Found {} entry points", entries.len());
        entries.len()
    }));

    // Thread 5: Transitive dependencies
    let graph5 = Arc::clone(&graph);
    let index_id5 = index_id.clone();
    handles.push(thread::spawn(move || {
        let transitive = graph5.transitive_dependencies(&index_id5).unwrap();
        println!(
            "Thread 5: index.ts has {} transitive dependencies",
            transitive.len()
        );
        transitive.len()
    }));

    // Wait for all threads to complete
    let mut results = vec![];
    for handle in handles {
        results.push(handle.join().unwrap());
    }

    println!("\nAll threads completed successfully!");
    println!("Results: {:?}", results);

    // Verify the graph is still accessible from the main thread
    let final_stats = graph.statistics()?;
    println!("\nFinal graph statistics:");
    println!("  Modules: {}", final_stats.module_count);
    println!("  Entry points: {}", final_stats.entry_point_count);
    println!(
        "  External dependencies: {}",
        final_stats.external_dependency_count
    );

    Ok(())
}
