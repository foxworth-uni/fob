//! Integration tests for async asset resolution.
//!
//! These tests verify the end-to-end async asset resolution chain works correctly
//! across different scenarios including:
//!
//! - Relative asset resolution (./asset.wasm)
//! - Node modules resolution (@scope/package/file.wasm)
//! - Multiple assets in single module
//! - Security validation (directory traversal prevention)
//! - Monorepo support
//!
//! # Educational Note: Integration vs Unit Tests
//!
//! Unlike the unit tests in asset_resolver.rs and asset_plugin.rs, these
//! integration tests verify the complete workflow from detection to resolution.
//! They ensure that all components work together correctly.

// Only run these tests on native platforms (not WASM)
#![cfg(not(target_family = "wasm"))]

use fob_core::builders::asset_resolver;
use fob_core::{FileMetadata, Runtime, RuntimeError, RuntimeResult};
use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Simple test runtime for integration tests.
///
/// Integration tests need their own copy of TestRuntime since they can't
/// access the crate's test_utils module (which is only available for unit tests).
/// This is the standard pattern in Rust for integration tests.
#[derive(Debug)]
struct TestRuntime {
    cwd: PathBuf,
}

impl TestRuntime {
    fn new(cwd: PathBuf) -> Self {
        Self { cwd }
    }
}

#[async_trait]
impl Runtime for TestRuntime {
    async fn read_file(&self, path: &Path) -> RuntimeResult<Vec<u8>> {
        std::fs::read(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn write_file(&self, path: &Path, content: &[u8]) -> RuntimeResult<()> {
        std::fs::write(path, content).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn metadata(&self, path: &Path) -> RuntimeResult<FileMetadata> {
        let metadata = std::fs::metadata(path).map_err(|e| RuntimeError::Io(e.to_string()))?;
        Ok(FileMetadata {
            size: metadata.len(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn resolve(&self, specifier: &str, from: &Path) -> RuntimeResult<PathBuf> {
        Ok(from.parent().unwrap_or(&self.cwd).join(specifier))
    }

    async fn create_dir(&self, path: &Path, recursive: bool) -> RuntimeResult<()> {
        if recursive {
            std::fs::create_dir_all(path).map_err(|e| RuntimeError::Io(e.to_string()))
        } else {
            std::fs::create_dir(path).map_err(|e| RuntimeError::Io(e.to_string()))
        }
    }

    async fn remove_file(&self, path: &Path) -> RuntimeResult<()> {
        std::fs::remove_file(path).map_err(|e| RuntimeError::Io(e.to_string()))
    }

    async fn read_dir(&self, path: &Path) -> RuntimeResult<Vec<String>> {
        let entries: Vec<String> = std::fs::read_dir(path)
            .map_err(|e| RuntimeError::Io(e.to_string()))?
            .filter_map(|entry| entry.ok().and_then(|e| e.file_name().to_str().map(String::from)))
            .collect();
        Ok(entries)
    }

    fn get_cwd(&self) -> RuntimeResult<PathBuf> {
        Ok(self.cwd.clone())
    }
}

/// Test basic relative asset resolution from a JavaScript module.
///
/// This verifies the most common case: a module using `new URL('./file.wasm', import.meta.url)`
/// to reference an asset in the same directory.
#[tokio::test]
async fn test_relative_asset_resolution() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create directory structure
    let src_dir = cwd.join("src");
    fs::create_dir(&src_dir).unwrap();

    // Create module and asset
    let module_path = src_dir.join("module.js");
    fs::write(&module_path, b"// module code").unwrap();

    let asset_path = src_dir.join("worker.wasm");
    fs::write(&asset_path, b"wasm binary").unwrap();

    // Resolve relative asset
    let resolved = asset_resolver::resolve_asset("./worker.wasm", &module_path, &cwd, &runtime)
        .await
        .expect("Failed to resolve relative asset");

    // Verify resolution is correct
    assert_eq!(
        resolved.canonicalize().unwrap(),
        asset_path.canonicalize().unwrap(),
        "Resolved path should match actual asset path"
    );

    // Verify file exists and can be read
    assert!(runtime.exists(&resolved));
    let content = runtime.read_file(&resolved).await.unwrap();
    assert_eq!(content, b"wasm binary");
}

/// Test asset resolution with parent directory traversal.
///
/// This tests the common pattern where assets are in a separate directory
/// from source files, e.g., `../assets/logo.png` from `src/component.js`.
#[tokio::test]
async fn test_parent_directory_asset_resolution() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create structure: project/src/module.js and project/assets/logo.png
    let src_dir = cwd.join("src");
    let assets_dir = cwd.join("assets");
    fs::create_dir(&src_dir).unwrap();
    fs::create_dir(&assets_dir).unwrap();

    let module_path = src_dir.join("component.js");
    fs::write(&module_path, b"// component").unwrap();

    let asset_path = assets_dir.join("logo.png");
    fs::write(&asset_path, b"PNG data").unwrap();

    // Resolve with parent traversal
    let resolved = asset_resolver::resolve_asset("../assets/logo.png", &module_path, &cwd, &runtime)
        .await
        .expect("Failed to resolve asset with parent traversal");

    assert_eq!(
        resolved.canonicalize().unwrap(),
        asset_path.canonicalize().unwrap()
    );
}

/// Test node_modules resolution for scoped packages.
///
/// This verifies that assets from npm packages can be resolved correctly,
/// testing the Node.js resolution algorithm.
#[tokio::test]
async fn test_node_modules_resolution() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create node_modules structure
    let pkg_dir = cwd.join("node_modules/@wasm-tool/converter/dist");
    fs::create_dir_all(&pkg_dir).unwrap();

    let asset_path = pkg_dir.join("converter.wasm");
    fs::write(&asset_path, b"wasm module").unwrap();

    // Create referrer module
    let src_dir = cwd.join("src");
    fs::create_dir(&src_dir).unwrap();
    let module_path = src_dir.join("index.js");
    fs::write(&module_path, b"// main module").unwrap();

    // Resolve from node_modules
    let resolved = asset_resolver::resolve_asset(
        "@wasm-tool/converter/dist/converter.wasm",
        &module_path,
        &cwd,
        &runtime,
    )
    .await
    .expect("Failed to resolve from node_modules");

    assert_eq!(
        resolved.canonicalize().unwrap(),
        asset_path.canonicalize().unwrap()
    );
}

/// Test multiple assets referenced from a single module.
///
/// This verifies that we can detect and resolve multiple asset references
/// in one module, which is common when modules use multiple WASM workers,
/// images, or other assets.
#[tokio::test]
async fn test_multiple_assets_single_module() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create assets
    let assets_dir = cwd.join("assets");
    fs::create_dir(&assets_dir).unwrap();

    let wasm_path = assets_dir.join("processor.wasm");
    fs::write(&wasm_path, b"wasm binary").unwrap();

    let image_path = assets_dir.join("icon.png");
    fs::write(&image_path, b"PNG data").unwrap();

    let font_path = assets_dir.join("font.woff2");
    fs::write(&font_path, b"WOFF2 data").unwrap();

    // Create module that references all assets
    let module_path = cwd.join("app.js");
    fs::write(&module_path, b"// app module").unwrap();

    // Resolve each asset
    let resolved_wasm =
        asset_resolver::resolve_asset("./assets/processor.wasm", &module_path, &cwd, &runtime)
            .await
            .expect("Failed to resolve WASM");

    let resolved_image =
        asset_resolver::resolve_asset("./assets/icon.png", &module_path, &cwd, &runtime)
            .await
            .expect("Failed to resolve image");

    let resolved_font =
        asset_resolver::resolve_asset("./assets/font.woff2", &module_path, &cwd, &runtime)
            .await
            .expect("Failed to resolve font");

    // Verify all are resolved correctly
    assert_eq!(
        resolved_wasm.canonicalize().unwrap(),
        wasm_path.canonicalize().unwrap()
    );
    assert_eq!(
        resolved_image.canonicalize().unwrap(),
        image_path.canonicalize().unwrap()
    );
    assert_eq!(
        resolved_font.canonicalize().unwrap(),
        font_path.canonicalize().unwrap()
    );
}

/// Test security validation: directory traversal prevention.
///
/// # Security Test
///
/// This is a critical security test that verifies we properly prevent
/// directory traversal attacks. An attacker might try to use paths like
/// `../../../../etc/passwd` to access files outside the project directory.
///
/// Our validation should:
/// 1. Detect when resolved path escapes project directory
/// 2. Check it's not in node_modules
/// 3. Check it's not in a monorepo workspace
/// 4. Reject the resolution with a security error
#[tokio::test]
async fn test_security_directory_traversal_prevention() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("project");
    fs::create_dir(&project_dir).unwrap();

    let cwd = project_dir.clone();
    let runtime = TestRuntime::new(cwd.clone());

    // Create file outside project directory
    let outside_file = temp.path().join("secret.txt");
    fs::write(&outside_file, b"sensitive data").unwrap();

    // Create module inside project
    let src_dir = cwd.join("src");
    fs::create_dir(&src_dir).unwrap();
    let module_path = src_dir.join("malicious.js");
    fs::write(&module_path, b"// trying to escape").unwrap();

    // Try to resolve path that escapes project (should fail)
    let result =
        asset_resolver::resolve_asset("../../secret.txt", &module_path, &cwd, &runtime).await;

    // Should return security violation error
    assert!(
        result.is_err(),
        "Directory traversal should be prevented"
    );

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("security") || error_msg.contains("outside"),
            "Error should mention security violation: {}",
            error_msg
        );
    }
}

/// Test security: ensure we can't access files outside project via absolute paths.
#[tokio::test]
async fn test_security_absolute_path_restriction() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path().join("project");
    fs::create_dir(&project_dir).unwrap();

    let cwd = project_dir.clone();
    let runtime = TestRuntime::new(cwd.clone());

    // Create file outside project
    let outside_file = temp.path().join("outside.wasm");
    fs::write(&outside_file, b"outside asset").unwrap();

    // Create module inside project
    let module_path = cwd.join("app.js");
    fs::write(&module_path, b"// app").unwrap();

    // Try to use absolute path to outside file (should fail)
    let result = asset_resolver::resolve_asset(
        outside_file.to_str().unwrap(),
        &module_path,
        &cwd,
        &runtime,
    )
    .await;

    assert!(result.is_err(), "Absolute path outside project should fail");
}

/// Test monorepo support: assets from sibling packages.
///
/// In monorepo setups, packages often need to reference assets from sibling
/// packages. This is legitimate and should be allowed when a monorepo root
/// is detected (pnpm-workspace.yaml, lerna.json, or package.json with workspaces).
#[tokio::test]
async fn test_monorepo_sibling_package_assets() {
    let temp = TempDir::new().unwrap();
    let monorepo_root = temp.path().to_path_buf();

    // Create monorepo marker (pnpm workspace)
    fs::write(
        monorepo_root.join("pnpm-workspace.yaml"),
        "packages:\n  - 'packages/*'\n",
    )
    .unwrap();

    // Create two packages
    let pkg_a_dir = monorepo_root.join("packages/app/src");
    let pkg_b_dir = monorepo_root.join("packages/shared/assets");
    fs::create_dir_all(&pkg_a_dir).unwrap();
    fs::create_dir_all(&pkg_b_dir).unwrap();

    // Asset in shared package
    let shared_asset = pkg_b_dir.join("shared.wasm");
    fs::write(&shared_asset, b"shared wasm").unwrap();

    // Module in app package referencing shared asset
    let app_module = pkg_a_dir.join("index.js");
    fs::write(&app_module, b"// app module").unwrap();

    // Set cwd to app package
    let cwd = monorepo_root.join("packages/app");
    let runtime = TestRuntime::new(cwd.clone());

    // Resolve cross-package reference
    let resolved = asset_resolver::resolve_asset(
        "../../shared/assets/shared.wasm",
        &app_module,
        &cwd,
        &runtime,
    )
    .await
    .expect("Should allow monorepo sibling package access");

    assert_eq!(
        resolved.canonicalize().unwrap(),
        shared_asset.canonicalize().unwrap()
    );
}

/// Test bare filename resolution (wasm-bindgen pattern).
///
/// wasm-bindgen generates code like `new URL('pkg_bg.wasm', import.meta.url)`
/// without the `./` prefix. This should resolve to a file in the same directory
/// as the referrer.
#[tokio::test]
async fn test_bare_filename_resolution() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create wasm-bindgen style output in node_modules
    let pkg_dir = cwd.join("node_modules/my-wasm-pkg");
    fs::create_dir_all(&pkg_dir).unwrap();

    let wasm_file = pkg_dir.join("pkg_bg.wasm");
    fs::write(&wasm_file, b"wasm content").unwrap();

    let js_file = pkg_dir.join("pkg.js");
    fs::write(&js_file, b"// wasm-bindgen generated").unwrap();

    // Resolve bare filename from JS file
    let resolved = asset_resolver::resolve_asset("pkg_bg.wasm", &js_file, &cwd, &runtime)
        .await
        .expect("Failed to resolve bare filename");

    assert_eq!(
        resolved.canonicalize().unwrap(),
        wasm_file.canonicalize().unwrap(),
        "Bare filename should resolve relative to referrer"
    );
}

/// Test asset size validation.
///
/// Ensures we can detect and reject assets that are too large,
/// preventing potential DoS attacks or resource exhaustion.
#[tokio::test]
async fn test_asset_size_validation() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create a small asset
    let small_asset = cwd.join("small.wasm");
    fs::write(&small_asset, vec![0u8; 1024]).unwrap(); // 1KB

    // Create a large asset
    let large_asset = cwd.join("large.wasm");
    fs::write(&large_asset, vec![0u8; 10 * 1024 * 1024]).unwrap(); // 10MB

    // Small asset should pass default limit (50MB)
    let size = asset_resolver::validate_asset_size(&small_asset, None, &runtime)
        .await
        .expect("Small asset should pass validation");
    assert_eq!(size, 1024);

    // Large asset should fail with small limit
    let result = asset_resolver::validate_asset_size(&large_asset, Some(1024 * 1024), &runtime).await;
    assert!(result.is_err(), "Large asset should fail size check");

    // Large asset should pass with large limit
    let size = asset_resolver::validate_asset_size(&large_asset, Some(50 * 1024 * 1024), &runtime)
        .await
        .expect("Large asset should pass with sufficient limit");
    assert_eq!(size, 10 * 1024 * 1024);
}

/// Test error case: asset not found.
///
/// Verifies proper error handling when an asset doesn't exist.
#[tokio::test]
async fn test_asset_not_found_error() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    let module_path = cwd.join("app.js");
    fs::write(&module_path, b"// app").unwrap();

    // Try to resolve non-existent asset
    let result =
        asset_resolver::resolve_asset("./nonexistent.wasm", &module_path, &cwd, &runtime).await;

    assert!(result.is_err(), "Should error for non-existent asset");

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("not found") || error_msg.contains("nonexistent"),
            "Error should indicate asset not found: {}",
            error_msg
        );
    }
}

/// Test nested node_modules resolution.
///
/// In complex projects, node_modules can be nested. We need to walk up
/// the directory tree to find the correct node_modules.
#[tokio::test]
async fn test_nested_node_modules_resolution() {
    let temp = TempDir::new().unwrap();
    let cwd = temp.path().to_path_buf();
    let runtime = TestRuntime::new(cwd.clone());

    // Create nested structure
    let nested_dir = cwd.join("src/components/features");
    fs::create_dir_all(&nested_dir).unwrap();

    // Put asset in root node_modules
    let pkg_dir = cwd.join("node_modules/pkg");
    fs::create_dir_all(&pkg_dir).unwrap();
    let asset_path = pkg_dir.join("asset.wasm");
    fs::write(&asset_path, b"asset content").unwrap();

    // Module deep in project structure
    let module_path = nested_dir.join("component.js");
    fs::write(&module_path, b"// component").unwrap();

    // Should walk up and find in root node_modules
    let resolved =
        asset_resolver::resolve_asset("pkg/asset.wasm", &module_path, &cwd, &runtime)
            .await
            .expect("Should find asset in root node_modules");

    assert_eq!(
        resolved.canonicalize().unwrap(),
        asset_path.canonicalize().unwrap()
    );
}
