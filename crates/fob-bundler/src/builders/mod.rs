//! Build APIs for Fob's unified bundling interface.
//!
//! This module provides a single, flexible entry point for all bundling operations
//! through the `BuildOptions` struct and `build()` function.
//!
//! # Examples
//!
//! ```no_run
//! use fob_bundler::{BuildOptions, Platform};
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! // Library mode: externalize dependencies
//! let result = BuildOptions::new("./src/index.ts")
//!     .externalize_from("package.json")
//!     .platform(Platform::Node)
//!     .build()
//!     .await?;
//!
//! // Bundle everything (standalone app)
//! let result = BuildOptions::new("./src/index.js")
//!     .platform(Platform::Browser)
//!     .build()
//!     .await?;
//!
//! // Multiple entries with code splitting
//! let result = BuildOptions::new_multiple(["./src/main.js", "./src/admin.js"])
//!     .bundle_together()
//!     .with_code_splitting()
//!     .outdir("dist")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub(crate) mod build_executor;
pub(crate) mod common;
pub(crate) mod runtime_file_plugin;
pub(crate) mod unified;

// Asset handling modules
pub mod asset_plugin;
pub mod asset_processor;
pub mod asset_registry;
pub mod asset_resolver;

// Re-export public API
pub use unified::{
    BuildOptions, BuildOutput, BuildResult, CodeSplittingConfig, EntryMode, EntryPoints,
    ExternalConfig, IncrementalConfig, MinifyLevel, build,
};

#[cfg(feature = "dts-generation")]
pub use unified::DtsOptions;

pub use common::plugin;
