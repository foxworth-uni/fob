#![cfg_attr(docsrs, feature(doc_cfg))]

//! # fob
//!
//! Fob - The unified API
//!
//! This crate re-exports all fob crates for convenience, providing a single
//! entry point to the fob ecosystem.

// Core types (always available)
pub use fob_core::*;

// Higher-level crates
pub use fob_analysis as analysis;
pub use fob_bundler as bundler;
pub use fob_config as config;
pub use fob_gen as codegen;
pub use fob_graph as graph;
