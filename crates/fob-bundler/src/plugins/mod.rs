//! Plugin system for fob-bundler.
//!
//! This module provides the plugin infrastructure, including:
//! - Plugin registry with execution phases
//! - Built-in plugins (DTS generation, etc.)

pub mod registry;

#[cfg(feature = "dts-generation")]
pub mod dts_emit;

pub use registry::{FobPlugin, PluginPhase, PluginRegistry};

#[cfg(feature = "dts-generation")]
pub use dts_emit::DtsEmitPlugin;
