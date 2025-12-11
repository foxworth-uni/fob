//! Fob bundler Ruby class

use crate::api::config::BundleConfig;
use crate::conversion::result::build_result_to_ruby_hash;
use crate::core::bundler::CoreBundler;
use crate::error::bundler_error_to_ruby_err;
use magnus::{Error, RHash, Ruby, Value};

/// Fob bundler for Ruby
#[magnus::wrap(class = "Fob::Bundler")]
pub struct Fob {
    bundler: CoreBundler,
}

impl Fob {
    /// Create a new bundler instance with full configuration control
    pub fn new(ruby: &Ruby, config: RHash) -> Result<Self, Error> {
        let bundle_config = BundleConfig::from_ruby_hash(ruby, config)?;
        let bundler = CoreBundler::new(bundle_config)
            .map_err(|e| Error::new(ruby.exception_runtime_error(), e))?;
        Ok(Self { bundler })
    }

    /// Bundle the configured entries and return detailed bundle information
    pub fn bundle(ruby: &Ruby, rb_self: &Self) -> Result<RHash, Error> {
        // Create a tokio runtime to run async code
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            Error::new(
                ruby.exception_runtime_error(),
                format!("Failed to create runtime: {}", e),
            )
        })?;

        // Block on the async bundle operation
        let result = rt
            .block_on(rb_self.bundler.bundle())
            .map_err(|e| bundler_error_to_ruby_err(e)(ruby))?;

        // Convert result to Ruby hash
        build_result_to_ruby_hash(ruby, &result)
    }
}

/// Build a standalone bundle (single entry, full bundling)
///
/// Best for: Applications, scripts, or single-file outputs.
pub fn bundle_entry(ruby: &Ruby, entry: String, options: Option<RHash>) -> Result<RHash, Error> {
    let opts_hash = options.unwrap_or_else(|| ruby.hash_new());
    let config_hash = ruby.hash_new();
    config_hash.aset(ruby.sym_new("entries"), vec![entry])?;

    // Merge options into config
    merge_options_into_config(ruby, &opts_hash, &config_hash)?;

    // Set entry_mode to shared
    config_hash.aset(ruby.sym_new("entry_mode"), "shared")?;

    let fob = Fob::new(ruby, config_hash)?;
    Fob::bundle(ruby, &fob)
}

/// Helper to merge options hash into config hash
fn merge_options_into_config(_ruby: &Ruby, opts: &RHash, config: &RHash) -> Result<(), Error> {
    use magnus::r_hash::ForEach;

    opts.foreach(|key: Value, val: Value| {
        config.aset(key, val)?;
        Ok(ForEach::Continue)
    })
}

/// Build a library (single entry, externalize dependencies)
///
/// Best for: npm packages, reusable libraries.
pub fn library(ruby: &Ruby, entry: String, options: Option<RHash>) -> Result<RHash, Error> {
    let opts_hash = options.unwrap_or_else(|| ruby.hash_new());
    let config_hash = ruby.hash_new();
    config_hash.aset(ruby.sym_new("entries"), vec![entry])?;

    // Merge options into config
    merge_options_into_config(ruby, &opts_hash, &config_hash)?;

    // Set entry_mode to shared and external_from_manifest to true
    config_hash.aset(ruby.sym_new("entry_mode"), "shared")?;
    config_hash.aset(ruby.sym_new("external_from_manifest"), true)?;

    let fob = Fob::new(ruby, config_hash)?;
    Fob::bundle(ruby, &fob)
}

/// Build an app with code splitting (multiple entries, unified output)
///
/// Best for: Web applications with multiple pages/routes.
pub fn app(ruby: &Ruby, entries: Vec<String>, options: Option<RHash>) -> Result<RHash, Error> {
    let opts_hash = options.unwrap_or_else(|| ruby.hash_new());
    let config_hash = ruby.hash_new();
    config_hash.aset(ruby.sym_new("entries"), entries)?;

    // Merge options into config
    merge_options_into_config(ruby, &opts_hash, &config_hash)?;

    // Set entry_mode to shared
    config_hash.aset(ruby.sym_new("entry_mode"), "shared")?;

    let fob = Fob::new(ruby, config_hash)?;
    Fob::bundle(ruby, &fob)
}

/// Build a component library (multiple entries, separate bundles)
///
/// Best for: UI component libraries, design systems.
pub fn components(
    ruby: &Ruby,
    entries: Vec<String>,
    options: Option<RHash>,
) -> Result<RHash, Error> {
    let opts_hash = options.unwrap_or_else(|| ruby.hash_new());
    let config_hash = ruby.hash_new();
    config_hash.aset(ruby.sym_new("entries"), entries)?;

    // Merge options into config
    merge_options_into_config(ruby, &opts_hash, &config_hash)?;

    // Set entry_mode to isolated and external_from_manifest to true
    config_hash.aset(ruby.sym_new("entry_mode"), "isolated")?;
    config_hash.aset(ruby.sym_new("external_from_manifest"), true)?;

    let fob = Fob::new(ruby, config_hash)?;
    Fob::bundle(ruby, &fob)
}
