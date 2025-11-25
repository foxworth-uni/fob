//! Dependency analysis example.
//!
//! This example demonstrates:
//! - Finding all dependencies of a module
//! - Finding all dependents (reverse dependencies)
//! - Finding transitive dependencies
//! - Detecting circular dependencies

use fob_graph::{Module, ModuleGraph, ModuleId, SourceType};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a graph with a dependency chain
    let graph = ModuleGraph::new()?;

    // Create modules: A -> B -> C -> D
    let a_id = ModuleId::new("src/a.ts")?;
    let b_id = ModuleId::new("src/b.ts")?;
    let c_id = ModuleId::new("src/c.ts")?;
    let d_id = ModuleId::new("src/d.ts")?;

    // Add modules
    for (id, path) in [
        (&a_id, "src/a.ts"),
        (&b_id, "src/b.ts"),
        (&c_id, "src/c.ts"),
        (&d_id, "src/d.ts"),
    ] {
        let module =
            Module::builder(id.clone(), PathBuf::from(path), SourceType::TypeScript).build();
        graph.add_module(module)?;
    }

    // Create dependency chain: A -> B -> C -> D
    graph.add_dependency(a_id.clone(), b_id.clone())?;
    graph.add_dependency(b_id.clone(), c_id.clone())?;
    graph.add_dependency(c_id.clone(), d_id.clone())?;

    // Also add: A -> C (skip level dependency)
    graph.add_dependency(a_id.clone(), c_id.clone())?;

    println!("Dependency chain: A -> B -> C -> D");
    println!("Also: A -> C (direct)\n");

    // Find direct dependencies of A
    println!("Direct dependencies of A:");
    let deps_a = graph.dependencies(&a_id)?;
    for dep in &deps_a {
        println!("  - {}", dep.path_string());
    }

    // Find direct dependents of C
    println!("\nDirect dependents of C (modules that depend on C):");
    let dependents_c = graph.dependents(&c_id)?;
    for dependent in &dependents_c {
        println!("  - {}", dependent.path_string());
    }

    // Find transitive dependencies (all dependencies, including indirect)
    println!("\nTransitive dependencies of A:");
    let transitive_deps = graph.transitive_dependencies(&a_id)?;
    for dep in &transitive_deps {
        println!("  - {}", dep.path_string());
    }

    // Check for circular dependencies by examining dependency chains
    println!("\nChecking for circular dependencies...");
    let chains_to_d = graph.dependency_chains_to(&d_id)?;
    let circular_chains: Vec<_> = chains_to_d.iter().filter(|c| c.has_cycle()).collect();
    if circular_chains.is_empty() {
        println!("  No circular dependencies found ✓");
    } else {
        println!(
            "  Found {} circular dependency chain(s):",
            circular_chains.len()
        );
        for chain in circular_chains {
            println!("    {}", chain.format_chain());
        }
    }

    // Find dependency chains to a specific module
    println!("\nDependency chains leading to D:");
    let chains_to_d = graph.dependency_chains_to(&d_id)?;
    for chain in chains_to_d {
        println!("  {}", chain.format_chain());
        if chain.has_cycle() {
            println!("    (circular)");
        }
    }

    Ok(())
}
