//! Framework-aware dead code analysis.
//!
//! This module provides a generic trait for marking exports as framework-used,
//! allowing external tools to implement framework-specific detection logic.
//!
//! # Overview
//!
//! Framework rules solve the problem of false-positive "unused export" warnings
//! for exports that frameworks consume through naming conventions rather than
//! explicit imports. Joy provides the infrastructure (the `FrameworkRule` trait),
//! while external tools (like Danny) provide the framework-specific knowledge.
//!
//! # Custom Rules
//!
//! ```rust,ignore
//! use fob_core::graph::{FrameworkRule, ModuleGraph};
//! use async_trait::async_trait;
//!
//! struct MyRule;
//!
//! #[async_trait]
//! impl FrameworkRule for MyRule {
//!     async fn apply(&self, graph: &ModuleGraph) -> Result<()> {
//!         let modules = graph.modules().await?;
//!         
//!         for module in modules {
//!             let mut updated = module.clone();
//!             let mut changed = false;
//!             
//!             for export in updated.exports.iter_mut() {
//!                 if export.name.starts_with("server_") {
//!                     export.mark_framework_used();
//!                     changed = true;
//!                 }
//!             }
//!             
//!             if changed {
//!                 graph.add_module(updated).await?;
//!             }
//!         }
//!         
//!         Ok(())
//!     }
//!
//!     fn name(&self) -> &'static str { "MyRule" }
//!     fn description(&self) -> &'static str { "Server actions" }
//!     fn clone_box(&self) -> Box<dyn FrameworkRule> { Box::new(MyRule) }
//! }
//!
//! // Apply custom rules via AnalyzeOptions
//! use fob_core::analysis::{analyze_with_options, AnalyzeOptions};
//!
//! let options = AnalyzeOptions {
//!     framework_rules: vec![Box::new(MyRule)],
//! };
//! let result = analyze_with_options(["src/index.tsx"], options).await?;
//! ```

mod trait_def;

pub use trait_def::FrameworkRule;
