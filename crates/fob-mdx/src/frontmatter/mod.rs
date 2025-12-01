//! Frontmatter parsing and types
//!
//! Handles extraction and parsing of YAML and TOML frontmatter blocks
//! from MDX documents during compilation.

mod parser;
mod types;

pub use parser::extract_frontmatter;
pub use types::{FrontmatterData, FrontmatterFormat};
