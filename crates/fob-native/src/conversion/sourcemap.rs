//! Source map mode conversion

use crate::types::SourceMapMode;
use fob_bundler::BuildOptions;

/// Convert source map mode and apply to BuildOptions
pub fn convert_sourcemap_mode(options: BuildOptions, mode: Option<SourceMapMode>) -> BuildOptions {
    match mode {
        Some(SourceMapMode::External) => options.sourcemap(true),
        Some(SourceMapMode::Inline) => options.sourcemap_inline(),
        Some(SourceMapMode::Hidden) => options.sourcemap_hidden(),
        Some(SourceMapMode::Disabled) | None => options.sourcemap(false),
    }
}
