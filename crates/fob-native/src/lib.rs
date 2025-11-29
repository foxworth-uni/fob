#![deny(clippy::all)]

//! Native Node.js bindings for Fob bundler core

pub mod api;
pub mod bundle_result;
pub mod conversion;
pub mod core;
pub mod error;
pub mod error_mapper;
pub mod runtime;
pub mod types;

// Re-export public API
pub use api::{BundleConfig, Fob, bundle_single, version};
pub use bundle_result::BundleResult;
pub use types::{OutputFormat, SourceMapMode};
