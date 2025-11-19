//! Ergonomic JavaScript code generation using OXC AST builders
//!
//! This crate provides a high-level, type-safe API for generating JavaScript code
//! using the OXC (Oxidation Compiler) AST infrastructure.
//!
//! # Features
//!
//! - **Type-safe AST building** - Leverage Rust's type system for correct JavaScript generation
//! - **Zero-copy string handling** - Efficient string interning via OXC's `Atom`
//! - **Ergonomic API** - Intuitive method names that mirror JavaScript syntax
//! - **Full module support** - Generate imports, exports, and ES modules
//! - **Modern JS features** - Arrow functions, template literals, destructuring, and more
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use fob_gen::JsBuilder;
//! use oxc_allocator::Allocator;
//! use oxc_ast::ast::Statement;
//!
//! let allocator = Allocator::default();
//! let js = JsBuilder::new(&allocator);
//!
//! // Build: const x = 42;
//! let stmt = js.const_decl("x", js.number(42.0));
//!
//! // Build: import React from 'react';
//! let import = js.import_default("React", "react");
//!
//! // Generate code
//! let code = js.program(vec![
//!     Statement::from(import),
//!     stmt,
//! ])?;
//! println!("{}", code);
//! # Ok::<(), fob_gen::GenError>(())
//! ```
//!
//! ## Building Complex Expressions
//!
//! ```rust
//! use fob_gen::JsBuilder;
//! use oxc_allocator::Allocator;
//!
//! let allocator = Allocator::default();
//! let js = JsBuilder::new(&allocator);
//!
//! // Build: console.log("Hello, world!")
//! let console_log = js.call(
//!     js.member(js.ident("console"), "log"),
//!     vec![js.arg(js.string("Hello, world!"))],
//! );
//! let stmt = js.expr_stmt(console_log);
//!
//! let code = js.program(vec![stmt])?;
//! # Ok::<(), fob_gen::GenError>(())
//! ```
//!
//! ## Arrow Functions and Arrays
//!
//! ```rust
//! use fob_gen::JsBuilder;
//! use oxc_allocator::Allocator;
//!
//! let allocator = Allocator::default();
//! let js = JsBuilder::new(&allocator);
//!
//! // Build: const double = x => x * 2;
//! let arrow = js.arrow_fn(
//!     vec!["x"],
//!     js.binary(
//!         js.ident("x"),
//!         oxc_ast::ast::BinaryOperator::Multiplication,
//!         js.number(2.0),
//!     ),
//! );
//! let stmt = js.const_decl("double", arrow);
//!
//! let code = js.program(vec![stmt])?;
//! # Ok::<(), fob_gen::GenError>(())
//! ```

mod builder;
mod dev_ui;
mod error;
mod format;
mod jsx;
mod program_builder;

#[cfg(feature = "parser")]
mod parser;

#[cfg(feature = "query-api")]
mod query;

#[cfg(feature = "transform-engine")]
mod transform;

#[cfg(feature = "fob_internal")]
mod internal;

pub use builder::JsBuilder;
pub use dev_ui::{HtmlBuilder, RouteSpec};
pub use error::{GenError, Result};
pub use format::{FormatOptions, IndentStyle, QuoteStyle};
pub use jsx::JsxBuilder;
pub use program_builder::ProgramBuilder;

#[cfg(feature = "parser")]
pub use parser::{parse, ParseDiagnostic, ParseOptions, ParsedProgram};

#[cfg(feature = "query-api")]
pub use query::{CallQuery, ExportQuery, ImportQuery, JsxQuery, QueryBuilder};

#[cfg(feature = "transform-engine")]
pub use transform::{TransformEngine, TransformOutput, TransformPass, TransformResult};

#[cfg(feature = "fob_internal")]
pub use internal::{AstMutations, DevInjection, ImportManipulation};

// Re-export commonly used OXC types for convenience
pub use oxc_allocator::Allocator;
pub use oxc_ast::ast::{BinaryOperator, LogicalOperator, UnaryOperator};
pub use oxc_span::Atom;
