//! Frontmatter parsing and types
//!
//! Handles extraction and parsing of YAML and TOML frontmatter blocks
//! from MDX documents during compilation.
//!
//! ## Props Support
//!
//! MDX frontmatter can include a `props:` section with data provider expressions:
//!
//! ```yaml
//! props:
//!   stars: github.repo("owner/name").stargazers_count @refresh=60s
//! ```

mod parser;
mod props;
mod props_parser;
mod types;

pub use parser::extract_frontmatter;
pub use props::{PropArg, PropDefinition, PropOptions, RefreshStrategy};
pub use props_parser::{PropParseError, parse_prop_expression};
pub use types::{FrontmatterData, FrontmatterFormat};
