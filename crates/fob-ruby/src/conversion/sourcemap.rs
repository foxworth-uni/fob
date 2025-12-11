//! Source map mode conversion

use fob_bundler::BuildOptions;

/// Convert source map mode (string representation of boolean | "inline" | "hidden" | "external") and apply to BuildOptions
/// Accepts: "true", "false", "inline", "hidden", "external", or None
pub fn convert_sourcemap_mode(
    options: BuildOptions,
    mode: Option<String>,
) -> Result<BuildOptions, String> {
    match mode.as_deref() {
        Some("true") | Some("external") => Ok(options.sourcemap(true)),
        Some("false") | None => Ok(options.sourcemap(false)),
        Some("inline") => Ok(options.sourcemap_inline()),
        Some("hidden") => Ok(options.sourcemap_hidden()),
        Some(other) => Err(format!(
            "Invalid sourcemap value '{}'. Expected: true, false, inline, hidden, external",
            other
        )),
    }
}
