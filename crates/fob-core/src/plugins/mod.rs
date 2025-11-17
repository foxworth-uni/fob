//! Build plugins for Fob bundler.

#[cfg(feature = "dts-generation")]
pub mod dts_emit;

#[cfg(feature = "dts-generation")]
pub use dts_emit::DtsEmitPlugin;
