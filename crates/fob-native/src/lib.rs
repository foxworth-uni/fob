#![deny(clippy::all)]

//! Native Node.js bindings for Fob bundler core
//!
//! All enum-like options (format, log_level, entry_mode) are exposed as strings
//! for better JavaScript/TypeScript ergonomics. Case-insensitive parsing is used.

pub mod api;
pub mod bundle_result;
pub mod conversion;
pub mod core;
pub mod error;
pub mod error_mapper;
pub mod runtime;
pub mod types;

// Re-export public API
pub use api::{BundleConfig, Fob, bundle_single, init_logging, init_logging_from_env, version};
pub use bundle_result::BundleResult;
