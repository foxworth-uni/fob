//! Unified build API for the Fob bundler.
//!
//! This module provides a configuration-based API for all bundling operations
//! using three orthogonal primitives:
//!
//! 1. **EntryMode**: Shared (entries can share chunks) vs Isolated (independent bundles)
//! 2. **CodeSplittingConfig**: Code splitting configuration (Option = on/off)
//! 3. **ExternalConfig**: External dependencies (None, List, FromManifest)
//!
//! # Examples
//!
//! ```no_run
//! use fob_bundler::BuildOptions;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! // Single entry standalone bundle
//! let result = BuildOptions::new("./src/index.js")
//!     .outfile("dist/bundle.js")
//!     .build()
//!     .await?;
//!
//! // Library: externalize dependencies
//! let result = BuildOptions::new("./src/index.ts")
//!     .externalize_from("package.json")
//!     .platform(fob_bundler::Platform::Node)
//!     .outfile("dist/index.js")
//!     .build()
//!     .await?;
//!
//! // App with code splitting
//! let result = BuildOptions::new_multiple(["./src/main.js", "./src/admin.js"])
//!     .bundle_together()
//!     .with_code_splitting()
//!     .outdir("dist")
//!     .build()
//!     .await?;
//!
//! // Component library: separate independent bundles
//! let result = BuildOptions::new_multiple(["./src/Button.tsx", "./src/Card.tsx"])
//!     .bundle_separately()
//!     .outdir("dist")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "dts-generation")]
mod dts;
mod entry;
mod minify;
mod options;
mod output;
pub mod primitives;

use crate::Result;

#[cfg(feature = "dts-generation")]
pub use dts::DtsOptions;
pub use entry::EntryPoints;
pub use minify::MinifyLevel;
pub use options::BuildOptions;
pub use output::{BuildOutput, BuildResult};
pub use primitives::{CodeSplittingConfig, EntryMode, ExternalConfig, IncrementalConfig};

/// Execute a build with the given options.
///
/// This is the main entry point for all bundling operations in Fob.
/// Use `BuildOptions` builder methods for ergonomic configuration.
///
/// # Examples
///
/// ```no_run
/// use fob_bundler::{build, BuildOptions};
///
/// # async fn example() -> fob_bundler::Result<()> {
/// // Using builder pattern
/// let result = BuildOptions::new("./src/index.js")
///     .externalize(["react", "react-dom"])
///     .build()
///     .await?;
///
/// // Or construct directly
/// let result = build(BuildOptions {
///     entry: fob_bundler::EntryPoints::Single("./src/index.js".into()),
///     ..Default::default()
/// }).await?;
/// # Ok(())
/// # }
/// ```
pub async fn build(options: BuildOptions) -> Result<BuildResult> {
    // Validate configuration
    options.validate()?;

    // Dispatch to build executor
    crate::builders::build_executor::execute_build(options).await
}
