use rustc_hash::FxHashMap;
use std::path::Path;

/// Entry point(s) for a build operation.
#[derive(Debug, Clone)]
pub enum EntryPoints {
    /// Single entry point.
    Single(String),

    /// Multiple entry points with automatic naming.
    Multiple(Vec<String>),

    /// Named entry points with custom output names.
    ///
    /// Keys are the output chunk names, values are the import paths.
    Named(FxHashMap<String, String>),
}

impl From<&str> for EntryPoints {
    fn from(entry: &str) -> Self {
        Self::Single(entry.to_string())
    }
}

impl From<String> for EntryPoints {
    fn from(entry: String) -> Self {
        Self::Single(entry)
    }
}

impl From<&Path> for EntryPoints {
    fn from(entry: &Path) -> Self {
        Self::Single(entry.to_string_lossy().into_owned())
    }
}

impl<S: Into<String>> From<Vec<S>> for EntryPoints {
    fn from(entries: Vec<S>) -> Self {
        Self::Multiple(entries.into_iter().map(Into::into).collect())
    }
}
