//! Output format conversion

use fob_bundler::OutputFormat as BundlerOutputFormat;

/// Convert format string to fob-bundler OutputFormat (case-insensitive)
/// Errors on invalid values instead of silently falling back.
pub fn convert_format(format: Option<&str>) -> Result<BundlerOutputFormat, String> {
    match format.map(|s| s.to_lowercase()).as_deref() {
        Some("esm") => Ok(BundlerOutputFormat::Esm),
        Some("cjs") => Ok(BundlerOutputFormat::Cjs),
        Some("iife") => Ok(BundlerOutputFormat::Iife),
        Some(other) => Err(format!(
            "Invalid format '{}'. Expected: esm, cjs, iife",
            other
        )),
        None => Ok(BundlerOutputFormat::Esm),
    }
}
