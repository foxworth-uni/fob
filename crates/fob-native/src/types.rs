//! Type definitions for fob-native

use napi_derive::napi;

/// Output format for bundled code
#[napi(string_enum)]
#[derive(Clone, Debug)]
pub enum OutputFormat {
    /// ES Module format
    Esm,
    /// CommonJS format
    Cjs,
    /// Immediately Invoked Function Expression format
    Iife,
}

/// Source map generation mode
#[napi(string_enum)]
#[derive(Clone, Debug)]
pub enum SourceMapMode {
    /// Generate external source map file (.map)
    External,
    /// Generate inline source map (data URI in bundle)
    Inline,
    /// Generate source map but don't reference it in bundle
    Hidden,
    /// Disable source map generation
    Disabled,
}

