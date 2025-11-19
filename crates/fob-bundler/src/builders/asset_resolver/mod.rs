//! # Asset Path Resolution for WASM and Native Platforms
//!
//! This module provides async asset resolution that works in both native
//! and WASM environments through the Runtime trait abstraction.
//!
//! ## Overview
//!
//! When bundling JavaScript/TypeScript code, modules often reference static assets
//! using patterns like `new URL('./asset.wasm', import.meta.url)`. This module
//! resolves these asset specifiers to their actual filesystem locations, handling:
//!
//! - Relative paths (`./file.wasm`, `../assets/logo.png`)
//! - Node modules resolution (`@pkg/file.wasm`, `pkg/assets/file.wasm`)
//! - Bare filenames (wasm-bindgen pattern: `file.wasm` without `./`)
//! - Monorepo support (cross-package references)
//! - Security validation (directory traversal prevention)
//!
//! ## WASM Compatibility
//!
//! All filesystem operations are abstracted through the [`Runtime`](crate::Runtime) trait:
//!
//! - **Native**: Uses `std::fs` via [`NativeRuntime`](crate::NativeRuntime)
//! - **WASM**: Bridges to JavaScript via `BrowserRuntime`
//!
//! This allows the same asset resolution logic to run in both environments.
//!
//! ## Path Canonicalization
//!
//! Platform-specific path handling ensures consistent behavior:
//!
//! - **Native**: Attempts `std::fs::canonicalize()` for real symlink resolution,
//!   falls back to path normalization if that fails (e.g., path doesn't exist yet)
//! - **WASM**: Uses manual normalization via the `path-clean` crate (no symlinks
//!   exist in the browser's virtual filesystem)
//!
//! The `path-clean` crate correctly handles `.` and `..` components on all platforms.
//!
//! ## Security
//!
//! The module implements multiple security layers:
//!
//! 1. **Directory Traversal Prevention**: Validates that resolved paths don't
//!    escape the project directory using `..` tricks
//! 2. **Allowed Directories**: Assets must be within:
//!    - The project directory (cwd)
//!    - `node_modules` directories
//!    - Monorepo workspace root (if detected)
//! 3. **Size Validation**: Optional size limits prevent DoS attacks via huge files
//!
//! ## Resolution Algorithm
//!
//! The resolution process follows this logic:
//!
//! 1. **Relative Paths** (`./` or `../`): Resolve relative to the referrer's directory
//! 2. **Absolute Paths** (`/`): Resolve directly, validate security
//! 3. **Package Paths** (contains `/` or `@scope/`): Search up directory tree for
//!    `node_modules/specifier`
//! 4. **Bare Filenames** (no path separators): Try relative to referrer first
//!    (wasm-bindgen pattern), fall back to node_modules
//!
//! ## Usage Examples
//!
//! ### Basic Resolution
//!
//! ```rust
//! use fob_bundler::builders::asset_resolver;
//! use fob_bundler::NativeRuntime;
//! use std::path::Path;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! let runtime = NativeRuntime;
//! let cwd = std::env::current_dir()?;
//! let referrer = cwd.join("src/module.js");
//!
//! // Resolve relative asset
//! let resolved = asset_resolver::resolve_asset(
//!     "./worker.wasm",
//!     &referrer,
//!     &cwd,
//!     &runtime,
//! ).await?;
//!
//! println!("Resolved to: {}", resolved.display());
//! # Ok(())
//! # }
//! ```
//!
//! ### Size Validation
//!
//! ```rust
//! use fob_bundler::builders::asset_resolver;
//! use fob_bundler::NativeRuntime;
//! use std::path::Path;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! let runtime = NativeRuntime;
//! let asset_path = Path::new("large_file.wasm");
//!
//! // Validate with 10MB limit
//! let size = asset_resolver::validate_asset_size(
//!     asset_path,
//!     Some(10 * 1024 * 1024),
//!     &runtime,
//! ).await?;
//!
//! println!("Asset size: {} bytes", size);
//! # Ok(())
//! # }
//! ```
//!
//! ## Educational Notes
//!
//! ### Why Async?
//!
//! Even though native filesystem operations are synchronous, we use async for:
//!
//! 1. **WASM Compatibility**: Browser APIs (fetch, FileSystem Access API) are async
//! 2. **Future Proofing**: Enables potential remote asset fetching
//! 3. **Uniform Interface**: Same API works everywhere
//!
//! On native platforms, the async overhead is negligible (just a future wrapper).
//!
//! ### Why Runtime Trait?
//!
//! The Runtime trait abstracts filesystem operations, allowing:
//!
//! 1. **Platform Independence**: Same code runs on native and WASM
//! 2. **Testability**: Easy to inject mock filesystem for testing
//! 3. **Flexibility**: Can swap implementations (e.g., in-memory FS for tests)
//!
//! This is a core pattern in Rust for cross-platform code.

mod security;
mod tests;
mod validation;

use crate::{Error, Result, Runtime};
use std::path::{Path, PathBuf};

pub use validation::validate_asset_size;

use security::canonicalize_path;
use security::validate_asset_security;

/// Resolve an asset specifier to an absolute filesystem path.
///
/// # Arguments
///
/// * `specifier` - The asset specifier (e.g., "./file.wasm", "../assets/logo.png")
/// * `referrer` - Absolute path to the module that references the asset
/// * `cwd` - Current working directory (project root)
/// * `runtime` - Runtime for filesystem operations
///
/// # Returns
///
/// Absolute, canonicalized path to the asset file
///
/// # Errors
///
/// Returns error if the asset cannot be resolved or doesn't exist
pub async fn resolve_asset(
    specifier: &str,
    referrer: &Path,
    cwd: &Path,
    runtime: &dyn Runtime,
) -> Result<PathBuf> {
    eprintln!("[ASSET_RESOLVE] Resolving asset:");
    eprintln!("[ASSET_RESOLVE]   specifier: '{}'", specifier);
    eprintln!("[ASSET_RESOLVE]   referrer: {}", referrer.display());
    eprintln!("[ASSET_RESOLVE]   cwd: {}", cwd.display());

    // Handle relative paths (./file or ../file)
    if specifier.starts_with('.') {
        eprintln!("[ASSET_RESOLVE]   Type: relative path");
        return resolve_relative(specifier, referrer, cwd, runtime).await;
    }

    // Handle absolute paths (rare, but possible)
    if specifier.starts_with('/') {
        eprintln!("[ASSET_RESOLVE]   Type: absolute path");
        let path = PathBuf::from(specifier);
        return validate_and_canonicalize(&path, cwd, runtime).await;
    }

    // For bare specifiers (no leading . or /), we need to distinguish between:
    // 1. Bare filenames that should resolve relative to the referrer (e.g., "file.wasm")
    // 2. Package specifiers that should resolve from node_modules (e.g., "@pkg/file.wasm" or "pkg/file.wasm")
    //
    // Strategy:
    // - If it looks like a simple filename (no "/" or starts with "@org/"), try relative first
    // - If relative fails or it clearly looks like a package path, try node_modules
    //
    // This handles wasm-bindgen's pattern: new URL('file.wasm', import.meta.url)
    eprintln!("[ASSET_RESOLVE]   Type: bare specifier");

    // Check if it's a simple filename (no path separators except possibly in @scope)
    let is_simple_filename = !specifier.contains('/')
        || (specifier.starts_with('@') && specifier.matches('/').count() == 1);

    if !is_simple_filename {
        eprintln!("[ASSET_RESOLVE]   Looks like package path, trying node_modules");
        return resolve_from_node_modules(specifier, referrer, cwd, runtime).await;
    }

    // Try resolving as relative to referrer first (wasm-bindgen case)
    eprintln!("[ASSET_RESOLVE]   Trying as relative filename");
    match resolve_relative(specifier, referrer, cwd, runtime).await {
        Ok(resolved) => {
            eprintln!("[ASSET_RESOLVE]   ✓ Resolved relative to referrer");
            Ok(resolved)
        }
        Err(_) => {
            // Fall back to node_modules lookup
            eprintln!("[ASSET_RESOLVE]   Failed relative, trying node_modules");
            resolve_from_node_modules(specifier, referrer, cwd, runtime).await
        }
    }
}

/// Resolve a relative path from the referrer's location.
async fn resolve_relative(
    specifier: &str,
    referrer: &Path,
    cwd: &Path,
    runtime: &dyn Runtime,
) -> Result<PathBuf> {
    // Get the directory containing the referrer
    // Check if referrer looks like a file (has an extension) rather than using is_file()
    // because the referrer path might not exist on disk yet (e.g., in tests or virtual files)
    let looks_like_file = referrer.extension().is_some();

    let base_dir = if looks_like_file {
        referrer.parent().unwrap_or(cwd)
    } else {
        referrer
    };

    eprintln!("[ASSET_RESOLVE]   base_dir: {}", base_dir.display());

    // Join and resolve
    let resolved = base_dir.join(specifier);
    eprintln!("[ASSET_RESOLVE]   joined path: {}", resolved.display());

    validate_and_canonicalize(&resolved, cwd, runtime).await
}

/// Resolve an asset from node_modules using Node.js resolution algorithm.
///
/// Walks up the directory tree looking for node_modules/specifier
async fn resolve_from_node_modules(
    specifier: &str,
    referrer: &Path,
    cwd: &Path,
    runtime: &dyn Runtime,
) -> Result<PathBuf> {
    // Start from the referrer's directory
    // Check if referrer looks like a file (has an extension)
    let looks_like_file = referrer.extension().is_some();

    let mut current = if looks_like_file {
        referrer.parent().unwrap_or(cwd)
    } else {
        referrer
    };

    eprintln!(
        "[ASSET_RESOLVE]   Starting search from: {}",
        current.display()
    );

    // Walk up the directory tree
    let mut attempts = 0;
    loop {
        let candidate = current.join("node_modules").join(specifier);
        eprintln!(
            "[ASSET_RESOLVE]   Attempt {}: checking {}",
            attempts,
            candidate.display()
        );

        if runtime.exists(&candidate) {
            eprintln!("[ASSET_RESOLVE]   ✓ Found!");
            return validate_and_canonicalize(&candidate, cwd, runtime).await;
        }

        // Move to parent directory
        match current.parent() {
            Some(parent) => {
                current = parent;
                attempts += 1;
            }
            None => {
                eprintln!("[ASSET_RESOLVE]   ✗ Not found in any node_modules");
                break;
            }
        }
    }

    Err(Error::AssetNotFound {
        specifier: specifier.to_string(),
        searched_from: referrer.display().to_string(),
    })
}

/// Validate and canonicalize an asset path.
///
/// # Security
///
/// - Prevents directory traversal attacks
/// - Ensures path is within project or node_modules
/// - Validates file exists
async fn validate_and_canonicalize(
    path: &Path,
    cwd: &Path,
    runtime: &dyn Runtime,
) -> Result<PathBuf> {
    eprintln!("[ASSET_RESOLVE]   Validating: {}", path.display());

    // Check if file exists
    if !runtime.exists(path) {
        eprintln!("[ASSET_RESOLVE]   ✗ File does not exist");
        return Err(Error::AssetNotFound {
            specifier: path.display().to_string(),
            searched_from: cwd.display().to_string(),
        });
    }

    eprintln!("[ASSET_RESOLVE]   ✓ File exists");

    // Canonicalize to resolve symlinks and relative components
    let canonical = canonicalize_path(path, runtime).await?;

    eprintln!("[ASSET_RESOLVE]   Canonical: {}", canonical.display());

    // Security: Validate path is safe
    validate_asset_security(&canonical, cwd, runtime).await?;

    eprintln!("[ASSET_RESOLVE]   ✓ Security validation passed");
    eprintln!(
        "[ASSET_RESOLVE]   Final resolved path: {}",
        canonical.display()
    );

    Ok(canonical)
}
