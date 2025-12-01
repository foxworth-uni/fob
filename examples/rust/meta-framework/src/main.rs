//! Example demonstrating meta-framework bundling with fob
//!
//! This shows how to build a simple meta-framework with:
//! - File-based routing discovery (scanning `app/routes/`)
//! - Code splitting across routes for optimal loading
//! - Path aliases for clean imports (`@` → `./app`)
//! - Multi-entry bundling pattern
//!
//! Meta-frameworks like Next.js, Remix, and SvelteKit follow this pattern:
//! they scan a directory structure to discover routes, then bundle each
//! route as a separate entry point with shared code extracted into chunks.

use anyhow::{Context, Result};
use fob_bundler::{BuildOptions, NativeRuntime};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 Meta-Framework Builder\n");

    // Discover route files from the app/routes directory
    let routes = discover_routes("app/routes")?;
    println!("📁 Discovered {} routes:", routes.len());
    for route_path in routes.keys() {
        println!("   • {}", route_path);
    }
    println!();

    // Build the framework with code splitting enabled
    let output_dir = build_framework(&routes).await?;

    // Print build results
    print_build_stats(&output_dir)?;

    println!("\n✅ Build complete! Output in: {}", output_dir.display());

    Ok(())
}

/// Discovers route files by scanning the routes directory.
/// Returns a map of route paths to their file paths.
///
/// For example: `/about` → `app/routes/about.tsx`
fn discover_routes(routes_dir: &str) -> Result<HashMap<String, PathBuf>> {
    let mut routes = HashMap::new();
    let routes_path = Path::new(routes_dir);

    if !routes_path.exists() {
        anyhow::bail!("Routes directory not found: {}", routes_dir);
    }

    for entry in std::fs::read_dir(routes_path)
        .with_context(|| format!("Failed to read routes directory: {}", routes_dir))?
    {
        let entry = entry?;
        let path = entry.path();

        // Only process .tsx files
        if path.extension().and_then(|s| s.to_str()) == Some("tsx") {
            let route_name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid route filename")?;

            // Convert file name to route path: index.tsx → /, about.tsx → /about
            let route_path = if route_name == "index" {
                "/".to_string()
            } else {
                format!("/{}", route_name)
            };

            routes.insert(route_path, path);
        }
    }

    Ok(routes)
}

/// Builds the meta-framework with code splitting across routes.
///
/// Each route becomes a separate entry point, and fob automatically
/// extracts shared code into common chunks for efficient loading.
async fn build_framework(routes: &HashMap<String, PathBuf>) -> Result<PathBuf> {
    let output_dir = PathBuf::from("dist");
    let route_files: Vec<PathBuf> = routes.values().cloned().collect();

    println!("🔨 Building with code splitting enabled...\n");

    // NativeRuntime provides access to the native filesystem for asset operations
    let runtime = Arc::new(NativeRuntime);

    // Configure build options for meta-framework pattern
    // Using the `app` preset which enables bundling and code splitting
    BuildOptions::app(route_files)
        .runtime(runtime)
        .outdir(output_dir.clone())
        .minify_level("identifiers")
        .path_alias("@", "./app") // Clean imports: import "@/router"
        .build()
        .await
        .context("Build failed")?;

    Ok(output_dir)
}

/// Prints statistics about the generated build artifacts.
fn print_build_stats(output_dir: &Path) -> Result<()> {
    let mut chunks = Vec::new();

    for entry in std::fs::read_dir(output_dir)
        .with_context(|| format!("Failed to read output directory: {}", output_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("js") {
            let size = entry.metadata()?.len();
            chunks.push((
                path.file_name().unwrap().to_string_lossy().to_string(),
                size,
            ));
        }
    }

    chunks.sort_by_key(|(name, _)| name.clone());

    println!("📦 Generated {} chunks:", chunks.len());
    for (name, size) in chunks {
        println!("   • {} ({} bytes)", name, size);
    }

    Ok(())
}
