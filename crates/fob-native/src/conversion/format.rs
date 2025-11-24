//! Output format conversion

use crate::types::OutputFormat;
use fob_bundler::OutputFormat as BundlerOutputFormat;

/// Convert NAPI OutputFormat to fob-bundler OutputFormat
pub fn convert_format(format: Option<OutputFormat>) -> BundlerOutputFormat {
    match format {
        Some(OutputFormat::Esm) => BundlerOutputFormat::Esm,
        Some(OutputFormat::Cjs) => BundlerOutputFormat::Cjs,
        Some(OutputFormat::Iife) => BundlerOutputFormat::Iife,
        None => BundlerOutputFormat::Esm,
    }
}

