use rustc_hash::FxHashMap;

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

