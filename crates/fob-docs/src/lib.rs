#![deny(clippy::all)]
#![cfg(not(target_family = "wasm"))]
// fob-docs is native-only and uses std::fs for documentation generation
#![allow(clippy::disallowed_methods)]

//! Documentation extraction and generation utilities for the Fob bundler.
//!
//! This crate provides:
//! - A lightweight documentation model for exported symbols.
//! - A parser that extracts JSDoc comments from JavaScript/TypeScript source files using OXC.
//! - Generators for Markdown and JSON output.
//! - A Rolldown plugin for integrating documentation emission into the bundling pipeline.

pub mod error;
pub mod extractor;
pub mod jsdoc;
pub mod model;

#[cfg(any(feature = "markdown", feature = "json"))]
pub mod generators;

#[cfg(feature = "rolldown-integration")]
pub mod plugin;

#[cfg(feature = "llm")]
pub mod llm;

pub mod writeback;

pub use error::{DocsError, Result};
pub use extractor::{DocsExtractor, ExtractOptions};
pub use model::{
    Documentation, ExportedSymbol, JsDocTag, ModuleDoc, ParameterDoc, SourceLocation, SymbolKind,
};

#[cfg(feature = "markdown")]
pub use generators::markdown::render_markdown;

#[cfg(feature = "json")]
pub use generators::json::render_json;

#[cfg(feature = "rolldown-integration")]
pub use plugin::{DocsEmitPlugin, DocsEmitPluginOptions, DocsPluginOutputFormat};

#[cfg(feature = "llm")]
pub use llm::{EnhancementMode, LlmConfig, LlmEnhancer};

pub use writeback::{DocsWriteback, MergeStrategy, WritebackReport};
