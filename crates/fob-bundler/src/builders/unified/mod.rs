//! Unified build API for the Fob bundler.
//!
//! This module provides a single, flexible configuration-based API for all bundling operations.
//!
//! # Examples
//!
//! ```no_run
//! use fob_bundler::BuildOptions;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! // Bundle a single file with all dependencies (app/component)
//! let result = BuildOptions::new("./src/index.js")
//!     .bundle(true)
//!     .build()
//!     .await?;
//!
//! // Library: externalize dependencies
//! let result = BuildOptions::new("./src/index.ts")
//!     .bundle(false)
//!     .platform(fob_bundler::Platform::Node)
//!     .external(["react", "react-dom"])
//!     .build()
//!     .await?;
//!
//! // App with code splitting
//! let result = BuildOptions::new_multiple(["./src/main.js", "./src/admin.js"])
//!     .bundle(true)
//!     .splitting(true)
//!     .outdir("dist")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "dts-generation")]
mod dts;
mod entry;
mod options;
mod output;

use crate::Result;

#[cfg(feature = "dts-generation")]
pub use dts::DtsOptions;
pub use entry::EntryPoints;
pub use options::BuildOptions;
pub use output::{BuildOutput, BuildResult};

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
///     .bundle(false)
///     .external(["react", "react-dom"])
///     .build()
///     .await?;
///
/// // Or construct directly
/// let result = build(BuildOptions {
///     entry: fob_bundler::EntryPoints::Single("./src/index.js".into()),
///     bundle: false,
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

