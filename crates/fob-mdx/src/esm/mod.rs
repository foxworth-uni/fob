//! ESM (ECMAScript Module) validation for MDX
//!
//! This module validates import/export statements in MDX files to ensure they're
//! syntactically correct before passing them through to the JavaScript compiler.

mod parser;
mod validator;

pub use parser::{extract_imported_names, get_default_export_name, has_named_exports, is_reexport};
pub use validator::validate_esm_syntax;
