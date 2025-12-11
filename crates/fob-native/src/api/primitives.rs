//! Composable primitives for bundler configuration.
//!
//! These types mirror the Rust core primitives and allow JavaScript/TypeScript
//! users to configure bundler behavior using the same composable model.
//!
//! Entry mode is now a string: "shared" | "isolated" (case-insensitive)

use napi_derive::napi;

/// Configuration for code splitting.
///
/// Code splitting extracts shared dependencies into separate chunks.
#[napi(object)]
#[derive(Clone, Debug)]
pub struct CodeSplittingConfig {
    /// Minimum chunk size in bytes (default: 20000 = 20KB)
    pub min_size: u32,
    /// Minimum number of entry points that must import the same module (default: 2)
    ///
    /// This was previously called `min_share_count` but `min_imports` is clearer
    /// about what's being counted.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_splitting_config_conversion() {
        let config = CodeSplittingConfig {
            min_size: 50_000,
            min_imports: 3,
        };
        let core_config: fob_bundler::CodeSplittingConfig = config.into();
        assert_eq!(core_config.min_size, 50_000);
        assert_eq!(core_config.min_imports, 3);
    }
}
