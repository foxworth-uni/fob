//! NAPI bindings for fob-native.
//!
//! This module contains the public API exposed to Node.js through NAPI.

mod bundler;
pub mod config;
mod functions;

pub use bundler::Fob;
pub use config::BundleConfig;
pub use functions::{bundle_single, version};

