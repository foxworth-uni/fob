//! Rolldown plugin implementation for bunny-mdx
//!
//! This module provides a real Rolldown plugin that integrates bunny-mdx MDX compilation
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
//! use fob_plugin_mdx::BunnyMdxPlugin;
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Use with your Rolldown bundler configuration
//! let plugin = Arc::new(BunnyMdxPlugin::new(PathBuf::from(".")));
//! # Ok(())
//! # }
//! ```

use anyhow::Context;
use bunny_mdx::{compile, MdxCompileOptions};
use fob_bundler::{
    HookLoadArgs, HookLoadOutput, HookLoadReturn, HookResolveIdArgs, HookResolveIdOutput,
    HookResolveIdReturn, ModuleType, Plugin, PluginContext,
};
use std::borrow::Cow;
use std::path::PathBuf;

/// Rolldown plugin that compiles MDX files to JSX
///
/// This plugin intercepts `.mdx` file loading and compiles them to JSX using bunny-mdx,
/// then returns the JSX to Rolldown for normal bundling.
///
/// # Architecture
///
/// ```text
/// .mdx file → load() hook → bunny_mdx::compile() → JSX → Rolldown parser → Bundle
/// ```
///
/// The plugin is async-compatible (required by Rolldown) but performs synchronous
/// compilation internally, which is acceptable for build-time transforms.
#[derive(Debug, Clone)]
pub struct BunnyMdxPlugin {
    /// Configuration options for MDX compilation
    options: MdxCompileOptions,
    /// Project root for resolving relative file paths
    project_root: PathBuf,
}

impl BunnyMdxPlugin {
    /// Create a new BunnyMdxPlugin with default options
    ///
    /// Default configuration includes:
    /// - All MDX features enabled (GFM, footnotes, math)
    /// - Default plugins (image optimization, heading IDs)
    /// - React 19 automatic JSX runtime
    ///
    /// # Arguments
    ///
    /// * `project_root` - The root directory of the project, used to resolve relative file paths
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_mdx::BunnyMdxPlugin;
    /// use std::path::PathBuf;
    ///
    /// let plugin = BunnyMdxPlugin::new(PathBuf::from("/path/to/project"));
    /// ```
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            options: MdxCompileOptions::new()
                .with_all_features()
                .with_default_plugins(),
            project_root,
        }
    }

    /// Create a new BunnyMdxPlugin with custom options
    ///
    /// Use this when you need fine-grained control over MDX compilation,
    /// such as disabling certain features or adding custom plugins.
    ///
    /// # Arguments
    ///
    /// * `options` - Custom MDX compilation options
    /// * `project_root` - The root directory of the project, used to resolve relative file paths
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_plugin_mdx::BunnyMdxPlugin;
    /// use bunny_mdx::MdxCompileOptions;
    /// use std::path::PathBuf;
    ///
    /// let mut opts = MdxCompileOptions::new();
    /// opts.gfm = true;
    /// opts.math = false;
    /// let plugin = BunnyMdxPlugin::with_options(opts, PathBuf::from("/path/to/project"));
    /// ```
    pub fn with_options(options: MdxCompileOptions, project_root: PathBuf) -> Self {
        Self {
            options,
            project_root,
        }
    }
}

impl Default for BunnyMdxPlugin {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl Plugin for BunnyMdxPlugin {
    /// Returns the plugin name for debugging and logging
    fn name(&self) -> Cow<'static, str> {
        "bunny-mdx".into()
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

            // Relative path - resolve against importer's directory (not project_root!)
            if let Some(importer_path) = &importer {
                if specifier.starts_with("./") || specifier.starts_with("../") {
                    let importer = std::path::Path::new(importer_path);
                    if let Some(importer_dir) = importer.parent() {
                        let resolved = importer_dir.join(&specifier);
                        if resolved.exists() {
                            return Ok(Some(HookResolveIdOutput {
                                id: resolved
                                    .canonicalize()
                                    .unwrap_or(resolved)
                                    .to_string_lossy()
                                    .into_owned()
                                    .into(),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }

            // Bare specifier or no importer - resolve against project_root
            let resolved = project_root.join(&specifier);
            if resolved.exists() {
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
    /// 3. Compiles MDX → JSX using bunny-mdx
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
        let options = self.options.clone();
        let project_root = self.project_root.clone();

        async move {
            // Only handle .mdx files
            if !id.ends_with(".mdx") {
                return Ok(None);
            }

            // Resolve the file path - handle both absolute and relative paths
            let file_path = if std::path::Path::new(&id).is_absolute() {
                PathBuf::from(&id)
            } else {
                project_root.join(&id)
            };

            // Read the MDX source file
            let source = std::fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read MDX file: {}", file_path.display()))?;

            // Configure compilation with file path for better error messages
            let mut opts = options;
            opts.filepath = Some(id.clone());

            // Compile MDX to JSX
            let result = compile(&source, opts)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = BunnyMdxPlugin::new(PathBuf::from("."));
        assert_eq!(plugin.name(), "bunny-mdx");
    }

    #[test]
    fn test_plugin_with_custom_options() {
        let mut opts = MdxCompileOptions::new();
        opts.gfm = true;
        let plugin = BunnyMdxPlugin::with_options(opts, PathBuf::from("."));
        assert_eq!(plugin.name(), "bunny-mdx");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = BunnyMdxPlugin::default();
        assert_eq!(plugin.name(), "bunny-mdx");
    }
}
