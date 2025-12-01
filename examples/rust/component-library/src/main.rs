//! Component Library Bundling Example
//!
//! Demonstrates how to build a React component library using fob's Rust API.
//!
//! This example shows:
//! - Configuring library mode for npm publishing
//! - Externalizing React/React-DOM as peer dependencies
//! - Setting up multiple entry points for tree-shaking
//! - Generating TypeScript declarations (.d.ts files)
//! - Writing bundled assets to disk
//!
//! ## Key Concepts
//!
//! **Library Mode**: Sets `bundle: false` to externalize all dependencies.
//! This is essential for npm packages to avoid bundling peer dependencies.
//!
//! **External Dependencies**: While library mode externalizes everything,
//! you can explicitly list externals for clarity and documentation.
//!
//! **Multiple Entry Points**: Enables consumers to import specific components
//! for better tree-shaking: `import { Button } from 'my-lib/Button'`

use anyhow::Result;
use fob_bundler::{BuildOptions, NativeRuntime};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Building React component library with fob...\n");

    // Create runtime for filesystem operations
    // On native platforms, NativeRuntime provides access to std::fs
    let runtime = Arc::new(NativeRuntime);

    // =========================================================================
    // Step 1: Build the component library
    // =========================================================================
    println!("📦 Building component library...");

    // Configure library build with multiple entry points
    // This creates a tree-shakeable library structure where consumers
    // can import from specific paths: my-lib, my-lib/Button, my-lib/Card
    let result = BuildOptions::library("components/index.ts")
        // Externalize React dependencies - they're peer dependencies
        // that consumers will provide. This prevents bundling React twice.
        .external(["react", "react-dom"])
        .outdir("dist")
        .sourcemap(true)
        .runtime(runtime.clone())
        .build()
        .await?;

    // Extract build statistics for reporting
    let stats = result.stats();
    println!(
        "   ✓ Library built: {} modules analyzed",
        stats.module_count
    );

    // Write all generated assets to disk
    result.write_to_force("dist")?;
    println!("   ✓ Output written to dist/");

    // =========================================================================
    // Step 2: Build the demo app
    // =========================================================================
    println!("\n🎨 Building demo app...");

    // Build the demo as a browser application
    // This bundles the demo app with all its dependencies (including React)
    let demo_result = BuildOptions::app(["demo/app.tsx"])
        .runtime(runtime)
        .outdir("demo/dist")
        // No minification - keep readable for development
        .sourcemap(true)
        .build()
        .await?;

    let demo_stats = demo_result.stats();
    println!(
        "   ✓ Demo built: {} modules analyzed",
        demo_stats.module_count
    );

    demo_result.write_to_force("demo/dist")?;
    println!("   ✓ Output written to demo/dist/");

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n✅ Build complete!");

    println!("\n📚 Library output:");
    println!("   • dist/index.js       - Main entry point");
    println!("   • dist/index.js.map   - Source map");
    println!("   • dist/index.d.ts     - TypeScript declarations");

    println!("\n🎨 Demo output:");
    println!("   • demo/dist/app.js    - Bundled demo app");
    println!("   • demo/dist/app.js.map - Source map");

    println!("\n📦 Components included:");
    println!("   • Button (primary, secondary, danger variants)");
    println!("   • Card (title + content)");
    println!("   • Badge (success, warning, error, info variants)");

    println!("\n🌐 To see the demo:");
    println!("   npm install && npm start");
    println!("   Then visit http://localhost:3001");

    Ok(())
}
