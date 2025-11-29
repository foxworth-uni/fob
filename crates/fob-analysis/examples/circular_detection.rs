//! Circular dependency detection example.
//!
//! This example demonstrates how to detect circular dependencies in a module graph.

use fob_analysis::Analyzer;

#[tokio::main]
async fn main() -> fob_core::Result<()> {
    let analysis = Analyzer::new().entry("src/index.ts").analyze().await?;

    // Find circular dependencies
    let circular = analysis.find_circular_dependencies()?;

    if circular.is_empty() {
        println!("No circular dependencies found!");
    } else {
        println!("Found {} circular dependency chains:", circular.len());
        for (i, chain) in circular.iter().enumerate() {
            println!("\nChain {}:", i + 1);
            println!("  {:?}", chain);
        }
    }

    // You can also get dependency chains to specific modules
    if let Ok(modules) = analysis.graph.modules() {
        for module in modules.iter().take(5) {
            // Get all dependency chains leading to this module
            if let Ok(chains) = analysis.dependency_chains_to(&module.id) {
                println!("\nDependency chains to {}: {}", module.id, chains.len());
            }
        }
    }

    Ok(())
}
