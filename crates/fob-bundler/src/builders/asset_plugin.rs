//! # Rolldown Plugin for Asset Detection and Emission
//!
//! This plugin hooks into Rolldown's transform phase to automatically detect, emit,
//! and rewrite asset references in JavaScript/TypeScript code.
//!
//! ## Overview
//!
//! When bundling JavaScript modules, developers often reference static assets like
//! WASM files, images, fonts, and other resources using the standard pattern:
//!
//! ```javascript
//! const wasmUrl = new URL('./file.wasm', import.meta.url);
//! const worker = new Worker(wasmUrl);
//! ```
//!
//! This plugin:
//!
//! 1. **Detects** asset references in code using regex pattern matching
//! 2. **Resolves** asset paths using the async [`asset_resolver`] module
//! 3. **Emits** assets through Rolldown's asset emission system
//! 4. **Rewrites** URLs in code to point to the emitted asset filenames
//!
//! The result is that assets are automatically included in the bundle output
//! with proper cache-busting filenames (e.g., `file-a1b2c3d4.wasm`).
//!
//! ## WASM Compatibility
//!
//! This plugin works on both native and WASM platforms through careful design:
//!
//! ### The Send Challenge
//!
//! Rolldown's `Plugin` trait requires `Future + Send` for all async methods,
//! but WASM futures are NOT Send (they use `Rc<RefCell<...>>` internally).
//!
//! ### The Solution: SendWrapper
//!
//! We use a conditional wrapper pattern:
//!
//! - **Native**: Futures are naturally Send, no wrapper needed
//! - **WASM**: Use [`SendWrapper`] which safely implements Send for WASM futures
//!
//! This is safe on WASM because:
//! - WASM is always single-threaded
//! - JavaScript is single-threaded
//! - There are no threads to send data between
//! - The Send trait is just a marker for compiler safety
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Rolldown Transform Phase                                    │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │
//!                  v
//! ┌─────────────────────────────────────────────────────────────┐
//! │ AssetDetectionPlugin::transform()                           │
//! │  - Checks if file is JS/TS                                  │
//! │  - Calls detect_assets() on source code                     │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │
//!                  v
//! ┌─────────────────────────────────────────────────────────────┐
//! │ detect_assets()                                              │
//! │  - Uses regex to find: new URL('...', import.meta.url)      │
//! │  - Calls asset_resolver::resolve_asset() for each match     │
//! │  - Returns list of (specifier, resolved_path) pairs         │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │
//!                  v
//! ┌─────────────────────────────────────────────────────────────┐
//! │ asset_resolver::resolve_asset()                              │
//! │  - Async resolution using Runtime trait                      │
//! │  - Security validation                                       │
//! │  - Returns absolute path to asset                            │
//! └────────────────┬────────────────────────────────────────────┘
//!                  │
//!                  v
//! ┌─────────────────────────────────────────────────────────────┐
//! │ Asset Emission & Code Rewriting                             │
//! │  - Read asset content via Runtime                            │
//! │  - Emit through ctx.emit_file()                              │
//! │  - Get final filename from Rolldown                          │
//! │  - Rewrite code: './file.wasm' -> 'file-hash.wasm'          │
//! │  - Register in AssetRegistry                                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust
//! use fob_bundler::builders::asset_plugin::AssetDetectionPlugin;
//! use fob_bundler::builders::asset_registry::AssetRegistry;
//! use fob_bundler::NativeRuntime;
//! use std::sync::Arc;
//! use std::path::PathBuf;
//!
//! # fn example() {
//! let registry = Arc::new(AssetRegistry::new());
//! let runtime = Arc::new(NativeRuntime);
//! let cwd = PathBuf::from("/project");
//! let extensions = vec![".wasm".to_string(), ".png".to_string()];
//!
//! let plugin = AssetDetectionPlugin::new(
//!     registry,
//!     cwd,
//!     extensions,
//!     runtime,
//! );
//!
//! // Add to Rolldown bundler
//! // bundler.add_plugin(Arc::new(plugin));
//! # }
//! ```
//!
//! ## Supported Asset Patterns
//!
//! The plugin detects these patterns (all quote styles):
//!
//! ```javascript
//! // Single quotes
//! new URL('./file.wasm', import.meta.url)
//!
//! // Double quotes
//! new URL("./file.wasm", import.meta.url)
//!
//! // Template literals
//! new URL(`./file.wasm`, import.meta.url)
//!
//! // Relative parent paths
//! new URL('../assets/logo.png', import.meta.url)
//!
//! // Package paths
//! new URL('@pkg/file.wasm', import.meta.url)
//!
//! // Bare filenames (wasm-bindgen)
//! new URL('pkg_bg.wasm', import.meta.url)
//! ```
//!
//! ## Educational Notes
//!
//! ### Why Regex Instead of AST?
//!
//! Currently, we use regex pattern matching to detect asset references rather than
//! full AST parsing. This is a pragmatic choice because:
//!
//! 1. **Simpler**: Regex is straightforward and maintainable
//! 2. **Faster**: No need to parse entire AST
//! 3. **Sufficient**: The pattern is well-defined and unlikely to have false positives
//! 4. **Future-Proof**: Can migrate to AST parsing when Rolldown plugin API stabilizes
//!
//! The regex accurately matches the standard `new URL(string, import.meta.url)` pattern
//! without risk of false positives from similar-looking code.
//!
//! ### Why Transform Phase?
//!
//! We hook into the transform phase (not resolve or load) because:
//!
//! 1. **Context**: We have the full module source code
//! 2. **Timing**: Module is already resolved, we know its location
//! 3. **Capability**: We can modify the code to rewrite URLs
//! 4. **Integration**: Natural fit with Rolldown's plugin architecture
//!
//! ### Async Everything
//!
//! Even though regex matching is synchronous, we use async throughout because:
//!
//! 1. **Asset Resolution**: Must be async for WASM (browser filesystem is async)
//! 2. **File Reading**: Runtime::read_file() is async
//! 3. **Consistency**: Uniform async interface
//!
//! This is the same principle as in [`asset_resolver`]: design for the most
//! constrained platform (WASM) and it works everywhere.

use super::asset_registry::AssetRegistry;
use super::asset_resolver;

/// Helper wrapper to make futures Send on WASM.
///
/// # Educational Note: WASM Send Safety
///
/// This wrapper exists because:
/// 1. Rolldown's Plugin trait requires `Future + Send` unconditionally
/// 2. WASM futures (using JsFuture) are NOT Send because they contain Rc<RefCell<...>>
/// 3. However, WASM is ALWAYS single-threaded, so Send is meaningless
///
/// ## Safety Invariant
///
/// It is safe to implement Send for this wrapper on WASM because:
/// - WASM has no threads (single-threaded execution model)
/// - JavaScript is single-threaded
/// - There are no threads to send data between
/// - The Send trait is just a marker that the compiler uses for safety
///
/// On native platforms, this wrapper is a simple pass-through that preserves
/// the actual Send bound of the inner future.
///
/// ## Why This Pattern?
///
/// This is a pragmatic solution to integrate with Rolldown's Plugin trait
/// which was designed for native/multi-threaded environments. In an ideal
/// world, Rolldown would have conditional Send bounds like we implemented
/// for Runtime, but until then, this wrapper is the correct solution.
#[cfg(target_family = "wasm")]
pub struct SendWrapper<F>(F);

#[cfg(target_family = "wasm")]
impl<F> SendWrapper<F> {
    fn new(future: F) -> Self {
        Self(future)
    }
}

#[cfg(target_family = "wasm")]
impl<F: std::future::Future> std::future::Future for SendWrapper<F> {
    type Output = F::Output;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // Safety: We're pinning through to the inner future
        unsafe { self.map_unchecked_mut(|s| &mut s.0).poll(cx) }
    }
}

// Safety: WASM is single-threaded, so this is safe even though F might not be Send
#[cfg(target_family = "wasm")]
unsafe impl<F> Send for SendWrapper<F> {}

// On native, no wrapper needed - futures are already Send
// This type alias exists for API consistency but is never used in native builds
#[cfg(not(target_family = "wasm"))]
type _SendWrapper<F> = F;
use rolldown_common::{EmittedAsset, ModuleType};
use rolldown_plugin::{
    HookTransformArgs, HookTransformOutput, HookTransformReturn, HookUsage, Plugin,
    SharedTransformPluginContext,
};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Rolldown plugin that detects and emits asset references during bundling.
///
/// Scans JavaScript/TypeScript code for `new URL(path, import.meta.url)` patterns,
/// emits the assets through Rolldown's asset system, and rewrites URLs to point
/// to the emitted assets.
#[derive(Debug)]
pub struct AssetDetectionPlugin {
    /// Shared asset registry (thread-safe)
    registry: Arc<AssetRegistry>,

    /// Working directory for path resolution
    cwd: PathBuf,

    /// Asset extensions to process
    extensions: Vec<String>,

    /// Runtime for filesystem operations (required for WASM compatibility)
    runtime: Arc<dyn crate::Runtime>,
}

impl AssetDetectionPlugin {
    /// Create a new asset detection plugin.
    ///
    /// # Arguments
    ///
    /// * `registry` - Shared asset registry
    /// * `cwd` - Current working directory for resolving paths
    /// * `extensions` - File extensions to treat as assets (e.g., [".wasm", ".png"])
    /// * `runtime` - Runtime for filesystem operations (required for WASM compatibility)
    pub fn new(
        registry: Arc<AssetRegistry>,
        cwd: impl AsRef<Path>,
        extensions: Vec<String>,
        runtime: Arc<dyn crate::Runtime>,
    ) -> Self {
        Self {
            registry,
            cwd: cwd.as_ref().to_path_buf(),
            extensions,
            runtime,
        }
    }
}

impl Plugin for AssetDetectionPlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("fob:asset-handler")
    }

    fn register_hook_usage(&self) -> HookUsage {
        HookUsage::Transform
    }

    fn transform(
        &self,
        ctx: SharedTransformPluginContext,
        args: &HookTransformArgs<'_>,
    ) -> impl std::future::Future<Output = HookTransformReturn> + Send {
        let module_id = args.id.to_string();
        let code = args.code.to_string();
        let module_type = args.module_type.clone();
        let registry = self.registry.clone();
        let cwd = self.cwd.clone();
        let should_process = self.extensions.clone();
        let runtime = self.runtime.clone();

        #[cfg(target_family = "wasm")]
        {
            SendWrapper::new(async move {
                Self::transform_impl(
                    module_id,
                    code,
                    module_type,
                    registry,
                    cwd,
                    should_process,
                    runtime,
                    ctx,
                )
                .await
            })
        }

        #[cfg(not(target_family = "wasm"))]
        {
            async move {
                Self::transform_impl(
                    module_id,
                    code,
                    module_type,
                    registry,
                    cwd,
                    should_process,
                    runtime,
                    ctx,
                )
                .await
            }
        }
    }
}

impl AssetDetectionPlugin {
    /// Implementation of the transform hook logic.
    ///
    /// # Educational Note: Separating Logic from FFI Boundaries
    ///
    /// This method contains the actual implementation, separated from the
    /// platform-specific wrapper concerns. This allows us to:
    /// 1. Share code between WASM and native builds
    /// 2. Test the logic independently of Send concerns
    /// 3. Keep the complexity of the wrapper pattern minimal
    #[allow(clippy::too_many_arguments)]
    async fn transform_impl(
        module_id: String,
        code: String,
        module_type: ModuleType,
        registry: Arc<AssetRegistry>,
        cwd: PathBuf,
        should_process: Vec<String>,
        runtime: Arc<dyn crate::Runtime>,
        ctx: SharedTransformPluginContext,
    ) -> HookTransformReturn {
        // Only process JavaScript/TypeScript
        if !matches!(
            module_type,
            ModuleType::Js | ModuleType::Jsx | ModuleType::Ts | ModuleType::Tsx
        ) {
            return Ok(None);
        }

        // Detect assets in the code
        let assets = match detect_assets(&code, &module_id, &cwd, runtime.as_ref()).await {
            Some(assets) if !assets.is_empty() => assets,
            _ => {
                return Ok(None);
            }
        };

        let mut modified_code = code.clone();
        let mut modified = false;

        for (specifier, resolved_path) in assets {
            // Check if this is an asset we want to handle
            if !should_process.iter().any(|ext| specifier.ends_with(ext)) {
                continue;
            }

            // Register in registry
            registry.register(resolved_path.clone(), module_id.clone(), specifier.clone());

            // Read asset content using runtime
            let content = match runtime.read_file(&resolved_path).await {
                Ok(c) => c,
                Err(e) => {
                    ctx.warn(rolldown_common::LogWithoutPlugin {
                        message: format!("Failed to read asset {:?}: {}", resolved_path, e),
                        ..Default::default()
                    });
                    continue;
                }
            };

            // Emit asset through Rolldown
            let reference_id = match ctx.emit_file(
                EmittedAsset {
                    name: resolved_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string()),
                    original_file_name: Some(resolved_path.to_string_lossy().into_owned()),
                    file_name: None, // Let Rolldown generate with hash
                    source: content.into(),
                },
                None,
                None,
            ) {
                Ok(id) => id,
                Err(e) => {
                    ctx.warn(rolldown_common::LogWithoutPlugin {
                        message: format!("Failed to emit asset {:?}: {}", resolved_path, e),
                        ..Default::default()
                    });
                    continue;
                }
            };

            // Get final filename from Rolldown
            let final_filename = match ctx.get_file_name(&reference_id) {
                Ok(name) => name,
                Err(e) => {
                    ctx.warn(rolldown_common::LogWithoutPlugin {
                        message: format!("Failed to get filename for asset: {}", e),
                        ..Default::default()
                    });
                    continue;
                }
            };

            // Update registry with final URL
            let url_path = format!("/{}", final_filename);
            registry.set_url_path(&resolved_path, url_path.clone());

            // Rewrite the URL in code (try both single and double quotes)
            let patterns = vec![
                format!("new URL('{}', import.meta.url)", specifier),
                format!("new URL(\"{}\", import.meta.url)", specifier),
                format!("new URL(`{}`, import.meta.url)", specifier),
            ];

            for pattern in patterns {
                if modified_code.contains(&pattern) {
                    let new_pattern = pattern.replace(&specifier, &final_filename);
                    modified_code = modified_code.replace(&pattern, &new_pattern);
                    modified = true;
                    break;
                }
            }
        }

        if modified {
            Ok(Some(HookTransformOutput {
                code: Some(modified_code),
                map: None,
                side_effects: None,
                module_type: None,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Detect asset references in module source code.
///
/// Returns a list of (specifier, resolved_path) pairs for each discovered asset.
///
/// NOTE: Simplified implementation using regex pattern matching.
/// TODO: Use full AST parsing once Rolldown plugin API is stabilized.
async fn detect_assets(
    source: &str,
    module_id: &str,
    cwd: &Path,
    runtime: &dyn crate::Runtime,
) -> Option<Vec<(String, PathBuf)>> {
    use regex::Regex;

    // Pattern: new URL('...', import.meta.url) or new URL("...", import.meta.url)
    let re =
        Regex::new(r#"new\s+URL\s*\(\s*['"`]([^'"`]+)['"`]\s*,\s*import\.meta\.url\s*\)"#).ok()?;

    let mut assets = Vec::new();
    let module_path = Path::new(module_id);

    let matches: Vec<_> = re.captures_iter(source).collect();

    for cap in matches.iter() {
        if let Some(specifier_match) = cap.get(1) {
            let specifier = specifier_match.as_str();

            // Try to resolve the asset
            if let Ok(resolved) =
                asset_resolver::resolve_asset(specifier, module_path, cwd, runtime).await
            {
                assets.push((specifier.to_string(), resolved));
            }
        }
    }

    if assets.is_empty() {
        None
    } else {
        Some(assets)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use crate::test_utils::TestRuntime;
    use std::fs;
    use tempfile::TempDir;

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_detect_new_url_pattern() {
        let code = r#"
            const wasmUrl = new URL('./file.wasm', import.meta.url);
            const module = await import(wasmUrl.href);
        "#;

        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create the asset file
        let asset_file = cwd.join("file.wasm");
        fs::write(&asset_file, b"test").unwrap();

        let module_id = cwd.join("index.js").display().to_string();

        let assets = detect_assets(code, &module_id, &cwd, &runtime).await;
        assert!(assets.is_some());

        let assets = assets.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].0, "./file.wasm");
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_detect_relative_parent() {
        let code = r#"
            const url = new URL('../assets/logo.png', import.meta.url);
        "#;

        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create directory structure
        let assets_dir = cwd.join("assets");
        fs::create_dir(&assets_dir).unwrap();
        let asset_file = assets_dir.join("logo.png");
        fs::write(&asset_file, b"test").unwrap();

        let src_dir = cwd.join("src");
        fs::create_dir(&src_dir).unwrap();
        let module_id = src_dir.join("index.js").display().to_string();

        let assets = detect_assets(code, &module_id, &cwd, &runtime).await;
        assert!(assets.is_some());

        let assets = assets.unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].0, "../assets/logo.png");
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_no_false_positives() {
        // Should NOT detect these
        let code = r#"
            // Regular URL construction
            const url1 = new URL('https://example.com');

            // Different second argument
            const url2 = new URL('./file', document.baseURI);

            // Not a URL constructor
            const other = new SomeClass('./file', import.meta.url);
        "#;

        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());
        let module_id = cwd.join("index.js").display().to_string();

        let assets = detect_assets(code, &module_id, &cwd, &runtime).await;
        assert!(assets.is_none() || assets.unwrap().is_empty());
    }

    /// Integration test: Verify full async chain works end-to-end
    /// This test verifies that:
    /// 1. detect_assets() is async and uses Runtime
    /// 2. Asset resolution works through the async chain
    /// 3. Multiple assets can be detected in one pass
    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_full_async_chain() {
        let code = r#"
            const wasmUrl = new URL('./module.wasm', import.meta.url);
            const imageUrl = new URL('./logo.png', import.meta.url);
            const fontUrl = new URL('./fonts/font.woff2', import.meta.url);
        "#;

        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create asset files
        fs::write(cwd.join("module.wasm"), b"wasm content").unwrap();
        fs::write(cwd.join("logo.png"), b"png content").unwrap();

        let fonts_dir = cwd.join("fonts");
        fs::create_dir(&fonts_dir).unwrap();
        fs::write(fonts_dir.join("font.woff2"), b"font content").unwrap();

        let module_id = cwd.join("index.js").display().to_string();

        // Test async detect_assets
        let assets = detect_assets(code, &module_id, &cwd, &runtime).await;
        assert!(assets.is_some(), "Should detect assets");

        let assets = assets.unwrap();
        assert_eq!(assets.len(), 3, "Should detect all 3 assets");

        // Verify all assets were resolved correctly
        let specifiers: Vec<_> = assets.iter().map(|(s, _)| s.as_str()).collect();
        assert!(specifiers.contains(&"./module.wasm"));
        assert!(specifiers.contains(&"./logo.png"));
        assert!(specifiers.contains(&"./fonts/font.woff2"));

        // Verify paths are absolute and exist
        for (_specifier, resolved_path) in &assets {
            assert!(
                resolved_path.is_absolute(),
                "Resolved path should be absolute: {}",
                resolved_path.display()
            );
            assert!(
                runtime.exists(resolved_path),
                "Asset should exist: {}",
                resolved_path.display()
            );
        }
    }
}
