//! Composable primitives for bundler configuration

use pyo3::Bound;
use pyo3::prelude::*;

/// Controls whether entry points share code or are isolated
#[derive(Clone, Debug, Default)]
pub enum EntryMode {
    /// Entries can share chunks (was: Unified)
    #[default]
    Shared,
    /// Each entry stands alone (was: Separate)
    Isolated,
}

impl EntryMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "shared" => Some(Self::Shared),
            "isolated" => Some(Self::Isolated),
            _ => None,
        }
    }
}

impl From<EntryMode> for fob_bundler::EntryMode {
    fn from(m: EntryMode) -> Self {
        match m {
            EntryMode::Shared => Self::Shared,
            EntryMode::Isolated => Self::Isolated,
        }
    }
}

/// Configuration for code splitting
#[derive(Clone, Debug)]
pub struct CodeSplittingConfig {
    pub min_size: u32,
    pub min_imports: u32,
}

impl Default for CodeSplittingConfig {
    fn default() -> Self {
        Self {
            min_size: 20_000,
            min_imports: 2,
        }
    }
}

impl From<CodeSplittingConfig> for fob_bundler::CodeSplittingConfig {
    fn from(c: CodeSplittingConfig) -> Self {
        Self {
            min_size: c.min_size,
            min_imports: c.min_imports,
        }
    }
}

pub fn register_primitives(_m: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
