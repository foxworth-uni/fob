//! Plugin system for fob-bundler.
//!
//! This module provides the plugin infrastructure, including:
//! - Plugin registry with execution phases
//! - Built-in plugins (DTS generation, etc.)

pub(crate) mod registry;

#[cfg(feature = "dts-generation")]
pub(crate) mod dts_emit;

pub(crate) use registry::{FobPlugin, PluginPhase, PluginRegistry};

#[cfg(feature = "dts-generation")]
pub(crate) use dts_emit::DtsEmitPlugin;
