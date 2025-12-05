//! Advanced Bundler Example
//!
//! This example demonstrates production-ready bundling patterns with fob-core.
//! Shows advanced features and real-world scenarios.
//!
//! ## What This Shows
//!
//! - Multiple entry points
//! - Code splitting across entries
//! - Multiple output formats (ESM, CJS, IIFE)
//! - External dependencies
//! - Path aliases for clean imports
//! - Different build configurations
//!
//! ## Key Concepts
//!
//! **Multiple Entry Points**: Build several files at once, extracting shared code
//! into common chunks for optimal loading.
//!
//! **Code Splitting**: Automatically extract shared dependencies into separate files
//! that can be cached and reused.
//!
//! **Output Formats**:
//! - ESM: Modern browsers and Node.js
//! - CJS: Traditional Node.js modules
//! - IIFE: Self-executing function for browsers (no module system needed)
//!
//! **External Dependencies**: Mark packages as external so they're not bundled.
//! Useful for peer dependencies or when using a CDN.
//!
//! **Path Aliases**: Use clean import paths like `@lib/math` instead of
//! relative paths like `../../lib/math`.

use anyhow::Result;
use fob_bundler::{BuildOptions, NativeRuntime, OutputFormat};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Advanced Bundler Example");
    println!("===========================\n");

    let runtime = Arc::new(NativeRuntime);

    // =========================================================================
    // Build 1: ESM with Code Splitting
    // =========================================================================
    println!("📦 Building ESM bundle with code splitting...");

    let esm_result = BuildOptions::new_multiple(["input/app.js", "input/worker.js"])
        .bundle(true)
        .runtime(runtime.clone())
        .outdir("output/esm")
        .format(OutputFormat::Esm)
        .external(["lodash"]) // Don't bundle lodash (assume it's from CDN or npm)
        .path_alias("@lib", "./input/lib") // Use @lib/ instead of ./lib/
        .splitting(true) // Enable code splitting
        .sourcemap(true)
        .minify_level("identifiers")
        .build()
        .await?;

    esm_result.write_to_force("output/esm")?;

    let esm_stats = esm_result.stats();
    println!("   ✓ ESM bundle complete");
    println!("     • Modules: {}", esm_stats.module_count);
    println!("     • Output: output/esm/");
    println!("     • Features: Code splitting, minified, source maps");
    println!();

    // =========================================================================
    // Build 2: CommonJS for Node.js
    // =========================================================================
    println!("📦 Building CommonJS bundle for Node.js...");

    let cjs_result = BuildOptions::new_multiple(["input/app.js"])
        .bundle(true)
        .runtime(runtime.clone())
        .outdir("output/cjs")
        .format(OutputFormat::Cjs)
        .path_alias("@lib", "./input/lib")
        .sourcemap(true)
        // No minification - keep readable for Node.js debugging
        .build()
        .await?;

    cjs_result.write_to_force("output/cjs")?;

    let cjs_stats = cjs_result.stats();
    println!("   ✓ CommonJS bundle complete");
    println!("     • Modules: {}", cjs_stats.module_count);
    println!("     • Output: output/cjs/");
    println!("     • Features: require/module.exports, unminified");
    println!();

    // =========================================================================
    // Build 3: IIFE for Browsers (No Module System)
    // =========================================================================
    println!("📦 Building IIFE bundle for browsers...");

    let iife_result = BuildOptions::new_multiple(["input/app.js"])
        .bundle(true)
        .runtime(runtime)
        .outdir("output/iife")
        .format(OutputFormat::Iife)
        .path_alias("@lib", "./input/lib")
        .sourcemap(true)
        .minify_level("identifiers")
        .build()
        .await?;

    iife_result.write_to_force("output/iife")?;

    let iife_stats = iife_result.stats();
    println!("   ✓ IIFE bundle complete");
    println!("     • Modules: {}", iife_stats.module_count);
    println!("     • Output: output/iife/");
    println!("     • Features: Self-executing, minified, no module system needed");
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("✅ All bundles complete!\n");

    println!("📁 Output Structure:");
    println!("   output/");
    println!("   ├── esm/          # ES modules with code splitting");
    println!("   │   ├── app.js    # Main app bundle");
    println!("   │   ├── worker.js # Worker bundle");
    println!("   │   └── shared-*.js # Shared code chunks");
    println!("   ├── cjs/          # CommonJS for Node.js");
    println!("   │   └── app.js    # CommonJS format");
    println!("   └── iife/         # IIFE for browsers");
    println!("       └── app.js");

    println!("\n💡 Usage Examples:");
    println!("   ESM:  import {{ calculate }} from './output/esm/app.js'");
    println!("   CJS:  const {{ calculate }} = require('./output/cjs/app.js')");
    println!("   IIFE: <script src=\"output/iife/app.js\"></script>");

    println!("\n🚀 Try it:");
    println!("   node output/cjs/app.js");

    Ok(())
}
