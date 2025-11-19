use std::path::PathBuf;

/// TypeScript declaration file generation options.
#[cfg(feature = "dts-generation")]
#[derive(Debug, Clone, Default)]
pub struct DtsOptions {
    /// Enable .d.ts generation (default: auto-detect from entry extension).
    pub emit: Option<bool>,

    /// Output directory for .d.ts files, relative to bundle output.
    ///
    /// If None, .d.ts files are emitted alongside .js files.
    pub outdir: Option<PathBuf>,

    /// Strip @internal JSDoc tags from declarations (default: false).
    pub strip_internal: bool,

    /// Generate .d.ts.map source maps (default: false).
    pub sourcemap: bool,
}
