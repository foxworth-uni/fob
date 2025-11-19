//! Basic Bundler Example
//!
//! This example demonstrates the fundamentals of bundling with fob-core.
//! Perfect for getting started with JavaScript bundling in Rust.
//!
//! ## What This Shows
//!
//! - Single entry point bundling
//! - ESM output format
//! - Source map generation
//! - Minification
//! - Build statistics
//!
//! ## Key Concepts
//!
//! **Entry Point**: The starting file that imports other modules.
//! The bundler follows imports to create a dependency graph.
//!
//! **Output Format**: ESM (ES Modules) is the modern JavaScript module format
//! supported natively in browsers and Node.js.
//!
//! **Source Maps**: Enable debugging of bundled code by mapping back to original source.
//!
//! **Minification**: Reduces file size by removing whitespace and shortening names.

use anyhow::Result;
use fob_bundler::{BuildOptions, NativeRuntime, OutputFormat};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("📦 Basic Bundler Example");
    println!("========================\n");

    // Create runtime for filesystem operations
    let runtime = Arc::new(NativeRuntime);

    println!("🔨 Building bundle...\n");

    // Configure and run the build
    // This is the simplest way to bundle JavaScript with fob
    let result = BuildOptions::app(["input/index.js"])
        .runtime(runtime)
        .outdir("output")
        .format(OutputFormat::Esm)
        .sourcemap(true)
        .minify(true)
        .build()
        .await?;

    // Write all generated files to disk
    result.write_to_force("output")?;

    // Extract and display build statistics
    let stats = result.stats();
    let cache = result.cache();

    println!("✅ Build complete!\n");
    println!("📊 Statistics:");
    println!("   • Modules analyzed: {}", stats.module_count);
    println!("   • Cache hits: {}/{}", cache.hits, cache.total_requests);
    println!("   • Output: output/index.js");
    println!("   • Source map: output/index.js.map");

    println!("\n💡 What happened:");
    println!("   1. Fob read input/index.js");
    println!("   2. Followed all import statements");
    println!("   3. Bundled {} modules into one file", stats.module_count);
    println!("   4. Minified the output");
    println!("   5. Generated source maps for debugging");

    println!("\n🚀 Try it:");
    println!("   node output/index.js");

    Ok(())
}
