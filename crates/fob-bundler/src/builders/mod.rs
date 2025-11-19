//! Build APIs for Fob's unified bundling interface.
//!
//! This module provides a single, flexible entry point for all bundling operations
//! through the `BuildOptions` struct and `build()` function.
//!
//! # Examples
//!
//! ```no_run
//! use fob_bundler::BuildOptions;
//!
//! # async fn example() -> fob_bundler::Result<()> {
//! // Library mode: externalize dependencies
//! let result = BuildOptions::library("./src/index.ts")
//!     .build()
//!     .await?;
//!
//! // Bundle everything
//! let result = BuildOptions::new("./src/index.js")
//!     .bundle(true)
//!     .build()
//!     .await?;
//!
//! // App with code splitting
//! let result = BuildOptions::app(["./src/main.js", "./src/admin.js"])
//!     .outdir("dist")
//!     .build()
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub(crate) mod build_executor;
pub(crate) mod common;
pub(crate) mod unified;
mod virtual_file_plugin;

// Asset handling modules
pub mod asset_registry;
pub mod asset_resolver;
pub mod asset_plugin;
pub mod asset_processor;

// Re-export public API
pub use unified::{build, BuildOptions, BuildOutput, BuildResult, EntryPoints};

#[cfg(feature = "dts-generation")]
pub use unified::DtsOptions;

pub use common::plugin;
