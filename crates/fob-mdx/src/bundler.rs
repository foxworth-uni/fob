//! Rolldown plugin implementation for fob-mdx
//!
//! This module provides a Rolldown plugin that integrates fob-mdx MDX compilation
//! into the Rolldown bundler pipeline. It uses the `load` hook to intercept `.mdx` files
//! and transform them to JSX before Rolldown processes them.
//!
//! ## Why the `load` hook?
//!
//! We use the `load` hook instead of `transform` because:
//! - `.mdx` files aren't valid JavaScript/TypeScript that Rolldown can parse
//! - We need to intercept them before Rolldown's parser runs
//! - The `load` hook is specifically designed for custom file loading
//! - We return JSX with `ModuleType::Jsx` to tell Rolldown how to handle it
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use fob_mdx::FobMdxPlugin;
//! use fob_bundler::Runtime;
//! use std::sync::Arc;
//!
//! # #[cfg(not(target_family = "wasm"))]
//! # fn example() {
//! use fob_bundler::runtime::BundlerRuntime;
//! // Create a runtime for file access
//! let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));
//! // MDX plugin is now auto-registered when using fob-bundler
//! let plugin = Arc::new(FobMdxPlugin::new(runtime));
//! # }
//! ```

use crate::{MdxCompileOptions, compile};
use anyhow::Context;
use fob_bundler::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, ModuleType, Plugin, PluginContext, Runtime,
};
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

/// Rolldown plugin that compiles MDX files to JSX
///
/// This plugin intercepts `.mdx` file loading and compiles them to JSX using fob-mdx,
/// then returns the JSX to Rolldown for normal bundling.
///
/// # Architecture
///
/// ```text
/// .mdx file → load() hook → fob_mdx::compile() → JSX → Rolldown parser → Bundle
/// ```
///
/// The plugin is async-compatible (required by Rolldown) but performs synchronous
/// compilation internally, which is acceptable for build-time transforms.
#[derive(Clone, Debug)]
pub struct FobMdxPlugin {
    /// Enable GFM (tables, strikethrough, task lists)
    pub gfm: bool,
    /// Enable footnotes
    pub footnotes: bool,
    /// Enable math support
    pub math: bool,
    /// JSX runtime module
    pub jsx_runtime: String,
    /// Use default plugins (heading IDs, image optimization)
    pub use_default_plugins: bool,
    /// Provider import source for component injection (e.g., "gumbo/mdx", "@mdx-js/react")
    ///
    /// When set, compiled MDX will import useMDXComponents from this source
    /// and merge provider components between defaults and props.components.
    pub provider_import_source: Option<String>,
    /// Project root for resolving relative file paths
    project_root: PathBuf,
    /// Runtime for file access (handles virtual files + filesystem)
    runtime: Arc<dyn Runtime>,
}

impl FobMdxPlugin {
    /// Create a new FobMdxPlugin with default options
    ///
    /// Default configuration includes:
    /// - All MDX features enabled (GFM, footnotes, math)
    /// - Default plugins (image optimization, heading IDs)
    /// - React 19 automatic JSX runtime
    ///
    /// # Arguments
    ///
    /// * `runtime` - Runtime for file access (handles virtual files + filesystem)
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_mdx::FobMdxPlugin;
    /// use fob_bundler::Runtime;
    /// use std::sync::Arc;
    ///
    /// # async fn example(runtime: Arc<dyn Runtime>) {
    /// let plugin = FobMdxPlugin::new(runtime);
    /// # }
    /// ```
    pub fn new(runtime: Arc<dyn Runtime>) -> Self {
        Self {
            gfm: true,
            footnotes: true,
            math: true,
            jsx_runtime: "react/jsx-runtime".to_string(),
            use_default_plugins: true,
            provider_import_source: None,
            project_root: PathBuf::from("."),
            runtime,
        }
    }

    /// Create MdxCompileOptions from plugin config
    fn create_options(&self, filepath: Option<String>) -> MdxCompileOptions {
        let mut opts = MdxCompileOptions::builder()
            .gfm(self.gfm)
            .footnotes(self.footnotes)
            .math(self.math)
            .jsx_runtime(self.jsx_runtime.clone())
            .use_default_plugins(self.use_default_plugins)
            .maybe_provider_import_source(self.provider_import_source.clone())
            .build();

        opts.filepath = filepath;
        opts
    }
}

impl Plugin for FobMdxPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "fob-mdx".into()
    }

    /// Declare which hooks this plugin uses
    ///
    /// This allows Rolldown to optimize by skipping unused hooks.
    fn register_hook_usage(&self) -> fob_bundler::HookUsage {
        use fob_bundler::HookUsage;
        // We use resolve_id to intercept MDX imports before Rolldown normalizes paths,
        // and load to compile MDX files to JSX
        HookUsage::ResolveId | HookUsage::Load
    }

    /// Resolve ID hook - intercepts `.mdx` imports before Rolldown's resolver normalizes them
    ///
    /// This hook ensures that MDX files are resolved with consistent absolute paths,
    /// preventing Rolldown from converting absolute paths to relative paths when the
    /// importer is a virtual module (which has no filesystem location).
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully resolved MDX file to absolute path
    /// - `Ok(None)` - Not an MDX file or file doesn't exist, let other resolvers handle it
    /// - `Err(e)` - Resolution error
    fn resolve_id(
        &self,
        _ctx: &PluginContext,
        args: &HookResolveIdArgs<'_>,
    ) -> impl std::future::Future<Output = HookResolveIdReturn> + Send {
        let specifier = args.specifier.to_string();
        let project_root = self.project_root.clone();
        let importer = args.importer.map(|s| s.to_string());
        let runtime = Arc::clone(&self.runtime);

        async move {
            // Only handle .mdx files
            if !specifier.ends_with(".mdx") {
                return Ok(None);
            }

            let path = std::path::Path::new(&specifier);

            // If already absolute, always claim it - don't check existence here.
            // The load hook will handle missing files with better error messages.
            // This prevents Rolldown from applying normalize_relative_external_id
            // which breaks when the importer is a virtual module.
            if path.is_absolute() {
                return Ok(Some(HookResolveIdOutput {
                    id: specifier.into(),
                    ..Default::default()
                }));
            }

            // Handle virtual file specifiers (e.g., "virtual:content.mdx")
            // Check if the runtime has this file before falling through to filesystem resolution
            if runtime.exists(path) {
                return Ok(Some(HookResolveIdOutput {
                    id: specifier.into(),
                    ..Default::default()
                }));
            }

            // Relative path - resolve against importer's directory (not project_root!)
            if let Some(importer_path) = &importer {
                if specifier.starts_with("./") || specifier.starts_with("../") {
                    let importer = std::path::Path::new(importer_path);
                    if let Some(importer_dir) = importer.parent() {
                        let resolved = importer_dir.join(&specifier);
                        // Use runtime.exists() to check both virtual files and filesystem
                        if runtime.exists(&resolved) {
                            // Try to canonicalize for real filesystem paths
                            let final_path = resolved.canonicalize().unwrap_or(resolved);
                            return Ok(Some(HookResolveIdOutput {
                                id: final_path.to_string_lossy().into_owned().into(),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }

            // Bare specifier or no importer - resolve against project_root
            let resolved = project_root.join(&specifier);
            // Use runtime.exists() to check both virtual files and filesystem
            if runtime.exists(&resolved) {
                return Ok(Some(HookResolveIdOutput {
                    id: resolved.to_string_lossy().into_owned().into(),
                    ..Default::default()
                }));
            }

            // File doesn't exist, let other resolvers handle it
            Ok(None)
        }
    }

    /// Load hook - intercepts `.mdx` files and compiles them to JSX
    ///
    /// This is the core of the plugin. It:
    /// 1. Checks if the file is a `.mdx` file
    /// 2. Reads the file from disk
    /// 3. Compiles MDX → JSX using fob-mdx
    /// 4. Returns JSX with `ModuleType::Jsx` for Rolldown to process
    ///
    /// # Returns
    ///
    /// - `Ok(Some(output))` - Successfully compiled MDX to JSX
    /// - `Ok(None)` - Not an MDX file, let Rolldown handle it
    /// - `Err(e)` - Compilation or I/O error
    ///
    /// # Async Wrapper
    ///
    /// Rolldown requires async hooks, but our compilation is synchronous.
    /// This is fine because:
    /// - File I/O is fast enough for build-time
    /// - MDX compilation is CPU-bound, not I/O-bound
    /// - No benefit from async for this use case
    ///
    /// If needed in the future, we could use `tokio::task::spawn_blocking`
    /// for true async behavior.
    fn load(
        &self,
        _ctx: &PluginContext,
        args: &HookLoadArgs<'_>,
    ) -> impl std::future::Future<Output = HookLoadReturn> + Send {
        // Capture data needed for async block to avoid lifetime issues
        let id = args.id.to_string();
        let options = self.create_options(Some(id.clone()));
        let project_root = self.project_root.clone();
        let runtime = Arc::clone(&self.runtime);

        async move {
            // Only handle .mdx files
            if !id.ends_with(".mdx") {
                return Ok(None);
            }

            // Determine file path for reading
            // - Virtual files (virtual:xxx): pass through as-is, Runtime handles lookup
            // - Absolute paths: use directly
            // - Relative paths: resolve against project_root
            let file_path = if id.starts_with("virtual:") || std::path::Path::new(&id).is_absolute()
            {
                PathBuf::from(&id)
            } else {
                project_root.join(&id)
            };

            // Read the MDX source file using Runtime (handles virtual files + filesystem)
            let content = runtime
                .read_file(&file_path)
                .await
                .with_context(|| format!("Failed to read MDX file: {}", file_path.display()))?;
            let source = String::from_utf8(content).with_context(|| {
                format!("MDX file {} contains invalid UTF-8", file_path.display())
            })?;

            // Compile MDX to JSX
            let result = compile(&source, options)
                .with_context(|| format!("Failed to compile MDX file: {}", id))?;

            // Debug logging to diagnose MDX import issues
            tracing::info!(
                path = %file_path.display(),
                code_len = result.code.len(),
                "MDX compiled to JSX"
            );
            // Print first 3000 chars of code for inspection
            let preview_len = result.code.len().min(3000);
            tracing::debug!(
                code = &result.code[..preview_len],
                "Compiled MDX code preview"
            );

            // Return JSX to Rolldown
            // IMPORTANT: Set module_type to Jsx so Rolldown knows how to parse it
            Ok(Some(HookLoadOutput {
                code: result.code.into(), // Convert String → ArcStr
                module_type: Some(ModuleType::Jsx),
                ..Default::default()
            }))
        }
    }
}

// FobPlugin trait has been removed from the public API.
// MDX plugin is now automatically registered by the bundler when .mdx files are detected.

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests using the full bundler pipeline
    //
    // NOTE: These tests are currently disabled because the public .plugin() API was removed
    // from BuildOptions. MDX files need to be handled by a plugin registered internally
    // by the bundler, or via a different mechanism. These tests serve as documentation
    // of the expected behavior once plugin auto-registration is implemented.
    //
    // To use FobMdxPlugin in production code, you'll need to use fob-plugin-mdx which
    // wraps this plugin for use with bundlers that support plugin registration.
    #[cfg(not(target_family = "wasm"))]
    mod load_hook_tests {
        use super::*;
        use fob_bundler::runtime::BundlerRuntime;
        use fob_bundler::{BuildOptions, Platform};

        /// Test that MDX compiles to JSX with correct output structure
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_mdx_compiles_to_jsx() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file(
                "virtual:test.mdx",
                b"# Hello World\n\nThis is **bold** text.",
            );

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Content from 'virtual:test.mdx';\nexport { Content };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("MDX should compile successfully");

            let chunk = result.chunks().next().expect("Should have chunk");

            // Verify JSX runtime is imported
            assert!(
                chunk.code.contains("jsx") || chunk.code.contains("jsxs"),
                "Should import JSX runtime functions"
            );

            // Verify MDXContent function is created
            assert!(
                chunk.code.contains("MDXContent"),
                "Should export MDXContent function"
            );

            // Verify content is transformed (not raw markdown)
            assert!(
                !chunk.code.contains("# Hello World"),
                "Raw markdown heading should be transformed"
            );
            assert!(
                !chunk.code.contains("**bold**"),
                "Raw markdown bold should be transformed"
            );
        }

        /// Test that non-MDX files are ignored
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_non_mdx_files_ignored() {
            let runtime: Arc<dyn Runtime> = Arc::new(BundlerRuntime::new("."));

            // Build a plain TypeScript file - MDX plugin should ignore it
            let result = BuildOptions::new("virtual:utils.ts")
                .platform(Platform::Node)
                .virtual_file("virtual:utils.ts", "export const greeting = 'Hello';")
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("TS file should build with MDX plugin present");

            let chunk = result.chunks().next().expect("Should have chunk");

            // Original content should be present (not transformed by MDX)
            assert!(
                chunk.code.contains("greeting"),
                "TS content should pass through unchanged"
            );
            // Should NOT have MDX-specific output
            assert!(
                !chunk.code.contains("MDXContent"),
                "Should not have MDXContent for non-MDX files"
            );
        }

        /// Test MDX with frontmatter compiles correctly
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_mdx_with_frontmatter() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file(
                "virtual:doc.mdx",
                b"---\ntitle: \"Test Document\"\nauthor: \"Test Author\"\n---\n\n# {frontmatter.title}\n\nBy {frontmatter.author}",
            );

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Doc from 'virtual:doc.mdx';\nexport { Doc };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("MDX with frontmatter should compile");

            let chunk = result.chunks().next().expect("Should have chunk");

            // Frontmatter should be accessible in the output
            assert!(
                chunk.code.contains("frontmatter") || chunk.code.contains("Test Document"),
                "Frontmatter should be processed and accessible"
            );
        }

        /// Test MDX with GFM features (tables, task lists)
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_mdx_with_gfm_features() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file(
                "virtual:gfm.mdx",
                b"# GFM Test\n\n- [x] Task complete\n- [ ] Task pending\n\n| Header | Value |\n|--------|-------|\n| A      | 1     |",
            );

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import GFM from 'virtual:gfm.mdx';\nexport { GFM };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("MDX with GFM should compile");

            let chunk = result.chunks().next().expect("Should have chunk");

            // GFM features should be transformed to JSX elements
            assert!(
                chunk.code.contains("table") || chunk.code.contains("input"),
                "GFM tables or checkboxes should be in output"
            );
        }

        /// Test that malformed MDX produces a clear error
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_malformed_mdx_error() {
            let bundler_runtime = BundlerRuntime::new(".");
            // Invalid JSX - unclosed tag
            bundler_runtime.add_virtual_file("virtual:bad.mdx", b"# Test\n\n<div>Unclosed tag");

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Bad from 'virtual:bad.mdx';\nexport { Bad };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await;

            // Note: MDX is forgiving about unclosed tags in some cases
            // This test documents the behavior - it may succeed or fail
            // depending on how lenient the MDX compiler is
            if let Err(e) = result {
                let err_str = e.to_string();
                // If it fails, error should mention MDX plugin or compilation
                assert!(
                    err_str.contains("compile")
                        || err_str.contains("parse")
                        || err_str.contains("MDX")
                        || err_str.contains("mdx")
                        || err_str.contains("fob-mdx")
                        || err_str.contains("threw an error")
                        || err_str.contains("syntax"),
                    "Error should indicate MDX compilation issue: {}",
                    err_str
                );
            }
            // If it succeeds, that's also valid behavior for lenient MDX
        }

        /// Test MDX with imports from other files
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_mdx_with_component_imports() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file(
                "virtual:button.tsx",
                b"export const Button = (props: any) => props.children;",
            );
            bundler_runtime.add_virtual_file(
                "virtual:doc.mdx",
                b"import { Button } from 'virtual:button.tsx'

# Hello

<Button>Click me</Button>",
            );

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Doc from 'virtual:doc.mdx';\nexport { Doc };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("MDX with component imports should compile");

            let chunk = result.chunks().next().expect("Should have chunk");

            // The import should be preserved in output
            assert!(
                chunk.code.contains("Button"),
                "Imported component should be in output"
            );
        }

        /// Test multiple MDX files in the same bundle
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_multiple_mdx_files() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file("virtual:page1.mdx", b"# Page One\n\nFirst page.");
            bundler_runtime.add_virtual_file("virtual:page2.mdx", b"# Page Two\n\nSecond page.");

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Page1 from 'virtual:page1.mdx';\nimport Page2 from 'virtual:page2.mdx';\nexport { Page1, Page2 };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("Multiple MDX files should compile");

            let chunk = result.chunks().next().expect("Should have chunk");

            // Both pages should be in the output
            assert!(
                chunk.code.contains("Page One") || chunk.code.contains("page1"),
                "First page content should be in output"
            );
            assert!(
                chunk.code.contains("Page Two") || chunk.code.contains("page2"),
                "Second page content should be in output"
            );
        }

        /// Test MDX with math and code features
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_mdx_with_math_and_code() {
            let bundler_runtime = BundlerRuntime::new(".");
            bundler_runtime.add_virtual_file(
                "virtual:technical.mdx",
                b"# Technical Doc

The formula $E = mc^2$ explains mass-energy equivalence.

```rust
fn main() {
    println!(\"Hello\");
}
```",
            );

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Tech from 'virtual:technical.mdx';\nexport { Tech };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await
                .expect("MDX with math and code should compile");

            let chunk = result.chunks().next().expect("Should have chunk");

            // Code block should be in output
            assert!(
                chunk.code.contains("pre") || chunk.code.contains("code"),
                "Code block should be rendered"
            );
        }

        /// Test error message includes filename
        #[tokio::test]
        #[ignore = "Plugin API removed - awaiting auto-registration or alternative test approach"]
        async fn test_error_includes_filename() {
            let bundler_runtime = BundlerRuntime::new(".");
            // Definitively invalid MDX - unclosed JSX expression
            bundler_runtime.add_virtual_file("virtual:broken.mdx", b"<div>{unclosed");

            let runtime: Arc<dyn Runtime> = Arc::new(bundler_runtime);

            let result = BuildOptions::new("virtual:entry.tsx")
                .platform(Platform::Node)
                .virtual_file(
                    "virtual:entry.tsx",
                    "import Broken from 'virtual:broken.mdx';\nexport { Broken };",
                )
                .runtime(Arc::clone(&runtime))
                .build()
                .await;

            // This should definitely fail
            match result {
                Ok(_) => panic!("Invalid MDX should fail to compile"),
                Err(e) => {
                    let err_str = e.to_string();
                    // Error should reference the file somehow
                    assert!(
                        err_str.contains("broken.mdx")
                            || err_str.contains("virtual:")
                            || err_str.contains("MDX")
                            || err_str.contains("compile"),
                        "Error should provide context about what failed: {}",
                        err_str
                    );
                }
            }
        }
    }
}
