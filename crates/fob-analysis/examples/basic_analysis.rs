//! Basic analysis example.
//!
//! This example demonstrates the simplest way to analyze a module graph.

use fob_analysis::Analyzer;

#[tokio::main]
async fn main() -> fob::Result<()> {
    // Create analyzer and configure entry point
    let analysis = Analyzer::new()
        .entry("src/index.ts") // Required: transitions to Configured state
        .analyze()
        .await?;

    // Get unused exports
    let unused = analysis.unused_exports()?;
    println!("Found {} unused exports", unused.len());

    // Get external dependencies
    let externals = analysis.external_dependencies()?;
    println!("Found {} external dependencies", externals.len());

    // Print analysis summary
    println!("\n{}", analysis);

    Ok(())
}
