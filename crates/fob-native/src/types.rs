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
