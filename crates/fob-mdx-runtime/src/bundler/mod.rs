//! Bundling module for runtime MDX bundling
//!
//! This module is only available when the "bundler" feature is enabled.
//! It provides the `bundle_mdx` function and related types for bundling
//! MDX files with their dependencies at runtime.

mod types;

pub use types::{BundleMdxOptions, BundleMdxResult};

use anyhow::{Context, Result};
use fob_bundler::runtime::BundlerRuntime;
use fob_bundler::{BuildOptions, BuildOutput, BundleOutput, OutputFormat, build};
use fob_mdx::compile;
use fob_plugin_mdx::FobMdxPlugin;
use std::path::PathBuf;
use std::sync::Arc;

/// Extract JavaScript code from a Rolldown bundle output
///
/// Searches for the first entry chunk in the bundle and returns its code.
fn extract_bundle_code(bundle: &BundleOutput) -> Result<String> {
    use rolldown_common::Output;

    // Find the first JavaScript chunk (should be our entry)
    bundle
        .assets
        .iter()
        .find_map(|asset| {
            if let Output::Chunk(chunk) = asset {
                Some(chunk.code.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow::anyhow!("No JavaScript chunk found in bundle output"))
}

/// Compile and bundle MDX at runtime
///
/// This is the main entry point for runtime MDX bundling. It takes MDX source code
/// and a map of virtual files, then compiles the MDX to JSX and bundles all imports
/// into a single executable JavaScript string.
///
/// # Arguments
///
/// * `options` - Configuration for MDX compilation and bundling
///
/// # Returns
///
/// * `Ok(BundleMdxResult)` - Bundled JavaScript code and metadata
/// * `Err` - Compilation or bundling error
///
/// # Example
///
/// ```rust,no_run
/// use fob_mdx_runtime::bundler::{bundle_mdx, BundleMdxOptions};
/// use fob_mdx::MdxCompileOptions;
/// use std::collections::HashMap;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let result = bundle_mdx(BundleMdxOptions {
///     source: "# Hello\n\nimport X from './x.js'\n\n<X />".to_string(),
///     files: HashMap::from([
///         ("./x.js".into(), "export default () => 'Hi'".into()),
///     ]),
///     mdx_options: None, // Uses sensible defaults (all features ON)
/// }).await?;
///
/// // result.code is ready to execute on client
/// println!("Bundle: {}", result.code);
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// This function performs bundling synchronously in the current task. For high-throughput
/// servers, consider:
///
/// - Caching bundled results (MDX source hash → bundle)
/// - Rate limiting bundle requests
/// - Using a task queue for bundling operations
///
/// # Errors
///
/// Returns error if:
/// - MDX compilation fails (syntax error, invalid JSX)
/// - Bundling fails (missing import, invalid JavaScript)
/// - File I/O fails (temporary directory creation)
pub async fn bundle_mdx(options: BundleMdxOptions) -> Result<BundleMdxResult> {
    // Step 1: Compile MDX to JSX (defaults: all features ON, default plugins ON)
    let mdx_opts = options.mdx_options.unwrap_or_default();

    let mdx_result = compile(&options.source, mdx_opts).context("Failed to compile MDX to JSX")?;

    // Step 2: Bundle using fob-core with virtual files
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut build_opts = BuildOptions::new("__mdx_entry__.jsx")
        .format(OutputFormat::Esm)
        .sourcemap_hidden();

    let runtime = Arc::new(BundlerRuntime::new(cwd.clone()));

    build_opts = build_opts
        .runtime(runtime.clone())
        .plugin(Arc::new(FobMdxPlugin::new(runtime)));

    // Add MDX entry as virtual file
    build_opts
        .virtual_files
        .insert("__mdx_entry__.jsx".to_string(), mdx_result.code.clone());

    // Add all user-provided virtual files
    for (path, content) in options.files {
        build_opts.virtual_files.insert(path, content);
    }

    let build_result = build(build_opts)
        .await
        .context("Failed to bundle MDX and dependencies")?;

    // Step 3: Extract bundled code from result
    let bundled_code = match build_result.output {
        BuildOutput::Single(bundle) => extract_bundle_code(&bundle)?,
        BuildOutput::Multiple(_) => {
            anyhow::bail!("Unexpected multiple bundle output for single MDX file")
        }
    };

    Ok(BundleMdxResult {
        code: bundled_code,
        frontmatter: mdx_result.frontmatter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_mdx_bundling() {
        let result = bundle_mdx(BundleMdxOptions::builder().source("# Hello World").build()).await;
        assert!(result.is_ok());

        let bundle = result.unwrap();
        assert!(!bundle.code.is_empty());
        assert!(bundle.code.contains("Hello World") || bundle.code.len() > 100);
    }

    #[tokio::test]
    async fn test_mdx_with_frontmatter() {
        let mdx = r#"---
title: Test Post
author: Joy
---

# Hello
"#;
        let result = bundle_mdx(BundleMdxOptions::builder().source(mdx).build())
            .await
            .unwrap();

        // Note: Frontmatter is extracted during MDX compilation
        // This test just verifies bundling succeeds with frontmatter present
        assert!(!result.code.is_empty());
        // Frontmatter extraction is tested in fob-mdx crate
    }

    #[tokio::test]
    async fn test_mdx_with_imports() {
        let mdx = r#"
import Button from './Button.jsx'

# Test

<Button>Click</Button>
"#;

        let button_component = r#"
export default function Button({children}) {
    return <button>{children}</button>
}
"#;

        let options = BundleMdxOptions::builder()
            .source(mdx)
            .build()
            .with_file("./Button.jsx", button_component);

        let result = bundle_mdx(options).await;
        assert!(
            result.is_ok(),
            "Bundling with imports should succeed: {:?}",
            result.err()
        );

        let bundle = result.unwrap();
        assert!(!bundle.code.is_empty());
        // Bundle should contain button logic
        assert!(bundle.code.len() > 200);
    }
}
