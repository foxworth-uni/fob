//! Rolldown plugin for emitting TypeScript declaration files (.d.ts)
//!
//! This module provides a Rolldown plugin that generates TypeScript declaration files
//! using OXC's isolated declarations feature. It works in conjunction with Rolldown's
//! built-in TypeScript support to emit `.d.ts` files alongside JavaScript output.
//!
//! ## Architecture
//!
//! ```text
//! TypeScript source → Rolldown (OXC transform) → JavaScript bundle
//!                              ↓
//!                     DtsEmitPlugin (OXC isolated declarations)
//!                              ↓
//!                         .d.ts files
//! ```
//!
//! ## How It Works
//!
//! 1. Rolldown transforms TypeScript to JavaScript using OXC
//! 2. This plugin uses the `generate_bundle` hook to run after transformation
//! 3. For each TypeScript module, it uses OXC's `IsolatedDeclarations` to generate .d.ts
//! 4. Generated .d.ts files are added as `OutputAsset` to the bundle
//!
//! ## Why `generate_bundle` hook?
//!
//! - Runs after all transformations are complete
//! - Has access to the full module graph
//! - Can add new assets to the bundle before writing
//! - Efficient - runs once per bundle, not per module

use anyhow::{Context, Result};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions};
use oxc_parser::Parser;
use oxc_span::SourceType as OxcSourceType;
use rolldown_common::{Output, OutputAsset};
use rolldown_plugin::{HookGenerateBundleArgs, HookNoopReturn, Plugin, PluginContext};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Plugin configuration for .d.ts generation
#[derive(Debug, Clone)]
pub struct DtsEmitPlugin {
    /// Strip @internal declarations from .d.ts files
    strip_internal: bool,
    /// Generate .d.ts.map source maps
    sourcemap: bool,
    /// Custom output directory for .d.ts files (relative to bundle output)
    dts_dir: Option<PathBuf>,
}

impl DtsEmitPlugin {
    /// Create a new DtsEmitPlugin with the given configuration
    ///
    /// # Arguments
    ///
    /// * `strip_internal` - Remove declarations marked with @internal JSDoc tag
    /// * `sourcemap` - Generate .d.ts.map files for IDE navigation
    /// * `dts_dir` - Optional custom directory for .d.ts files
    ///
    /// # Example
    ///
    /// ```rust
    /// use fob_bundler::plugins::DtsEmitPlugin;
    ///
    /// let plugin = DtsEmitPlugin::new(
    ///     true,  // strip_internal
    ///     true,  // sourcemap
    ///     Some("types".into())  // dts_dir
    /// );
    /// ```
    pub fn new(strip_internal: bool, sourcemap: bool, dts_dir: Option<PathBuf>) -> Self {
        Self {
            strip_internal,
            sourcemap,
            dts_dir,
        }
    }
}

impl Plugin for DtsEmitPlugin {
    fn name(&self) -> Cow<'static, str> {
        "fob-dts-emit".into()
    }

    fn register_hook_usage(&self) -> rolldown_plugin::HookUsage {
        use rolldown_plugin::HookUsage;
        HookUsage::GenerateBundle
    }

    /// Generate .d.ts files and add them to the bundle
    ///
    /// This hook runs after all transformations and before writing to disk.
    /// It iterates through chunks, finds TypeScript modules, generates declarations,
    /// and adds them as assets to the bundle.
    fn generate_bundle(
        &self,
        _ctx: &PluginContext,
        args: &mut HookGenerateBundleArgs<'_>,
    ) -> impl std::future::Future<Output = HookNoopReturn> + Send {
        let strip_internal = self.strip_internal;
        let sourcemap = self.sourcemap;
        let dts_dir = self.dts_dir.clone();

        async move {
            let mut dts_assets = Vec::new();

            // Iterate through all outputs in the bundle
            for output in args.bundle.iter() {
                // We're only interested in chunks (not existing assets)
                if let Output::Chunk(chunk) = output {
                    // For library bundles, we typically have one main chunk
                    // Check all modules in this chunk for TypeScript sources
                    for module_id in &chunk.modules.keys {
                        // Only process TypeScript files
                        if !is_typescript_module(module_id.as_ref()) {
                            continue;
                        }

                        // Read the original TypeScript source
                        // Note: module_id is the file path
                        // SAFETY: This uses std::fs which is NOT WASM-compatible
                        // This crate has guards to prevent WASM compilation
                        let source = match std::fs::read_to_string(module_id.as_ref()) {
                            Ok(s) => s,
                            Err(e) => {
                                // If we can't read the file, skip it but don't fail the whole build
                                eprintln!(
                                    "Warning: Failed to read TypeScript file for .d.ts generation: {} - {}",
                                    module_id.as_ref(),
                                    e
                                );
                                continue;
                            }
                        };

                        // Generate .d.ts content using OXC
                        let dts_content =
                            match generate_dts(&source, module_id.as_ref(), strip_internal) {
                                Ok(content) => content,
                                Err(e) => {
                                    // If generation fails, warn but continue
                                    eprintln!(
                                        "Warning: Failed to generate .d.ts for {}: {}",
                                        module_id.as_ref(),
                                        e
                                    );
                                    continue;
                                }
                            };

                        // Compute output filename for the .d.ts file
                        let dts_filename = compute_dts_filename(
                            &chunk.filename,
                            module_id.as_ref(),
                            dts_dir.as_deref(),
                        );

                        // Create an OutputAsset for the .d.ts file
                        let asset = OutputAsset {
                            names: vec![],
                            original_file_names: vec![module_id.to_string()],
                            filename: dts_filename.into(),
                            source: dts_content.into(),
                        };

                        dts_assets.push(Output::Asset(Arc::new(asset)));

                        // TODO: Generate .d.ts.map if sourcemap is enabled
                        // This requires capturing source maps from OXC codegen
                        if sourcemap {
                            // Placeholder for future implementation
                            // Would need to use CodeGenerator with sourcemap enabled
                        }
                    }
                }
            }

            // Add all generated .d.ts files to the bundle
            args.bundle.extend(dts_assets);

            Ok(())
        }
    }
}

/// Check if a module path is a TypeScript file
fn is_typescript_module(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext, "ts" | "tsx" | "mts" | "cts"))
        .unwrap_or(false)
}

/// Generate .d.ts content from TypeScript source using OXC
///
/// # Arguments
///
/// * `source` - TypeScript source code
/// * `file_path` - Path to the source file (for error messages)
/// * `strip_internal` - Whether to strip @internal declarations
///
/// # Returns
///
/// The generated .d.ts file content as a String
///
/// # Errors
///
/// Returns an error if parsing or transformation fails
fn generate_dts(source: &str, file_path: &str, strip_internal: bool) -> Result<String> {
    // Create OXC allocator (required for all OXC operations)
    let allocator = Allocator::default();

    // Determine source type from file extension
    let source_type = OxcSourceType::from_path(file_path)
        .with_context(|| format!("Invalid TypeScript file: {}", file_path))?;

    // Parse the TypeScript source
    let parser = Parser::new(&allocator, source, source_type);
    let parse_result = parser.parse();

    // Check for parse errors
    if !parse_result.errors.is_empty() {
        let error_messages: Vec<String> = parse_result
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        anyhow::bail!(
            "Failed to parse TypeScript file {}: {}",
            file_path,
            error_messages.join(", ")
        );
    }

    // Create isolated declarations transformer
    let options = IsolatedDeclarationsOptions { strip_internal };
    let isolated_dts = IsolatedDeclarations::new(&allocator, options);

    // Transform to declarations
    let dts_result = isolated_dts.build(&parse_result.program);

    // Check for transformation errors
    if !dts_result.errors.is_empty() {
        let error_messages: Vec<String> = dts_result
            .errors
            .iter()
            .map(|e| format!("{:?}", e))
            .collect();
        anyhow::bail!(
            "Errors generating declarations for {}: {}",
            file_path,
            error_messages.join(", ")
        );
    }

    // Generate .d.ts string from AST
    let codegen = Codegen::new();
    let generated = codegen.build(&dts_result.program);

    Ok(generated.code)
}

/// Compute the output filename for a .d.ts file
///
/// # Arguments
///
/// * `chunk_filename` - The JavaScript chunk filename (e.g., "index.js")
/// * `module_id` - The original TypeScript module path
/// * `dts_dir` - Optional custom directory for .d.ts files
///
/// # Returns
///
/// The output path for the .d.ts file (e.g., "types/index.d.ts")
fn compute_dts_filename(chunk_filename: &str, _module_id: &str, dts_dir: Option<&Path>) -> String {
    // Get the base name from the chunk filename
    // e.g., "index.js" → "index"
    let base = Path::new(chunk_filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("index");

    let filename = format!("{}.d.ts", base);

    // If a custom directory is specified, prepend it
    if let Some(dir) = dts_dir {
        format!("{}/{}", dir.display(), filename)
    } else {
        filename
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_typescript_module() {
        assert!(is_typescript_module("index.ts"));
        assert!(is_typescript_module("component.tsx"));
        assert!(is_typescript_module("module.mts"));
        assert!(is_typescript_module("types.cts"));
        assert!(!is_typescript_module("index.js"));
        assert!(!is_typescript_module("index.jsx"));
        assert!(!is_typescript_module("style.css"));
    }

    #[test]
    fn test_compute_dts_filename() {
        // Without custom directory
        assert_eq!(
            compute_dts_filename("index.js", "src/index.ts", None),
            "index.d.ts"
        );

        // With custom directory
        assert_eq!(
            compute_dts_filename("index.js", "src/index.ts", Some(Path::new("types"))),
            "types/index.d.ts"
        );

        assert_eq!(
            compute_dts_filename("bundle.js", "src/lib.ts", Some(Path::new("dist/types"))),
            "dist/types/bundle.d.ts"
        );
    }

    #[test]
    fn test_generate_dts_basic() {
        let source = r#"
export function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;

        let result = generate_dts(source, "test.ts", false);
        assert!(result.is_ok());

        let dts = result.unwrap();
        assert!(dts.contains("export"));
        assert!(dts.contains("function greet"));
        assert!(dts.contains("string"));
    }

    #[test]
    fn test_generate_dts_with_strip_internal() {
        let source = r#"
/** @internal */
export function _internalFn(): void {}

export function publicFn(): void {}
"#;

        // With strip_internal = true
        let result = generate_dts(source, "test.ts", true);
        assert!(result.is_ok());
        let dts = result.unwrap();
        assert!(!dts.contains("_internalFn"));
        assert!(dts.contains("publicFn"));

        // With strip_internal = false
        let result = generate_dts(source, "test.ts", false);
        assert!(result.is_ok());
        let dts = result.unwrap();
        assert!(dts.contains("_internalFn"));
        assert!(dts.contains("publicFn"));
    }
}
