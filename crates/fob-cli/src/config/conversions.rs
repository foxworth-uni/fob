use crate::config::types::*;

// Conversion implementations: CLI enums -> Config enums

impl From<crate::cli::Format> for Format {
    fn from(f: crate::cli::Format) -> Self {
        match f {
            crate::cli::Format::Esm => Format::Esm,
            crate::cli::Format::Cjs => Format::Cjs,
            crate::cli::Format::Iife => Format::Iife,
        }
    }
}

impl From<crate::cli::Platform> for Platform {
    fn from(p: crate::cli::Platform) -> Self {
        match p {
            crate::cli::Platform::Browser => Platform::Browser,
            crate::cli::Platform::Node => Platform::Node,
        }
    }
}

impl From<crate::cli::SourceMapMode> for SourceMapMode {
    fn from(s: crate::cli::SourceMapMode) -> Self {
        match s {
            crate::cli::SourceMapMode::Inline => SourceMapMode::Inline,
            crate::cli::SourceMapMode::External => SourceMapMode::External,
            crate::cli::SourceMapMode::Hidden => SourceMapMode::Hidden,
        }
    }
}

// EsTarget conversion removed - config::EsTarget is now a re-export of cli::EsTarget
