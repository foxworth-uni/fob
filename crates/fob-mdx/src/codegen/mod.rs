//! JSX code generation
//!
//! Converts markdown AST nodes to JSX code strings with proper escaping
//! and React runtime integration.

mod context;
mod escape;
mod jsx_value;
mod jsx_writer;
mod renderer;

pub use context::{CodegenContext, TableContext};
pub use escape::{escape_js_string, is_valid_identifier};
pub use jsx_value::JsValue;
pub use renderer::{mdast_to_jsx, mdast_to_jsx_with_options};
