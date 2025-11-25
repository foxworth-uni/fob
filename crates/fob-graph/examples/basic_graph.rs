//! Basic graph construction and queries example.
//!
//! This example demonstrates:
//! - Creating a new ModuleGraph
//! - Adding modules to the graph
//! - Adding dependencies between modules
//! - Querying dependencies and dependents
//! - Checking entry points

use fob_graph::{Module, ModuleGraph, ModuleId, SourceType};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new empty graph
    let graph = ModuleGraph::new()?;

    // Create module IDs
    let index_id = ModuleId::new("src/index.ts")?;
    let utils_id = ModuleId::new("src/utils.ts")?;
    let api_id = ModuleId::new("src/api.ts")?;

    // Build modules using the builder pattern
    let index_module = Module::builder(
        index_id.clone(),
        PathBuf::from("src/index.ts"),
        SourceType::TypeScript,
    )
    .entry(true)
    .build();

    let utils_module = Module::builder(
        utils_id.clone(),
        PathBuf::from("src/utils.ts"),
        SourceType::TypeScript,
    )
    .build();

    let api_module = Module::builder(
        api_id.clone(),
        PathBuf::from("src/api.ts"),
        SourceType::TypeScript,
    )
    .build();

    // Add modules to the graph
    graph.add_module(index_module)?;
    graph.add_module(utils_module)?;
    graph.add_module(api_module)?;

    // Add dependencies: index depends on utils and api
    graph.add_dependency(index_id.clone(), utils_id.clone())?;
    graph.add_dependency(index_id.clone(), api_id.clone())?;
    // api also depends on utils
    graph.add_dependency(api_id.clone(), utils_id.clone())?;

    // Query dependencies
    println!("Dependencies of index.ts:");
    let deps = graph.dependencies(&index_id)?;
    for dep in deps {
        println!("  - {}", dep.path_string());
    }

    // Query dependents (reverse dependencies)
    println!("\nModules that depend on utils.ts:");
    let dependents = graph.dependents(&utils_id)?;
    for dependent in dependents {
        println!("  - {}", dependent.path_string());
    }

    // Check entry points
    println!("\nEntry points:");
    let entry_points = graph.entry_points()?;
    for entry in entry_points {
        println!("  - {}", entry.path_string());
    }

    // Get graph statistics
    let stats = graph.statistics()?;
    println!("\nGraph statistics:");
    println!("  Total modules: {}", stats.module_count);
    println!("  Entry points: {}", stats.entry_point_count);
    println!(
        "  External dependencies: {}",
        stats.external_dependency_count
    );

    Ok(())
}
