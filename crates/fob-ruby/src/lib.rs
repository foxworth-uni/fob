//! Ruby bindings for Fob bundler core
//!
//! This module provides Magnus bindings that mirror the Node.js and Python APIs,
//! allowing Ruby users to bundle JavaScript/TypeScript code using Fob.

mod api;
mod conversion;
mod core;
mod error;
mod runtime;
mod types;

use magnus::{Error, Module, Object, Ruby, function, method};

/// Ruby module initialization
#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("Fob")?;

    // Register error class
    error::register_error_class(ruby, module)?;

    // Register Fob::Bundler class
    let bundler_class = module.define_class("Bundler", ruby.class_object())?;
    // new takes 1 arg (config hash) - Ruby adds self implicitly
    bundler_class.define_singleton_method("new", function!(api::bundler::Fob::new, 1))?;
    // bundle takes 0 args - self is the Fob instance
    bundler_class.define_method("bundle", method!(api::bundler::Fob::bundle, 0))?;

    // Register convenience module methods (presets)
    // bundle_entry takes 2 args (entry, options)
    module.define_singleton_method("bundle_entry", function!(api::bundler::bundle_entry, 2))?;
    // library takes 2 args (entry, options)
    module.define_singleton_method("library", function!(api::bundler::library, 2))?;
    // app takes 2 args (entries array, options)
    module.define_singleton_method("app", function!(api::bundler::app, 2))?;
    // components takes 2 args (entries array, options)
    module.define_singleton_method("components", function!(api::bundler::components, 2))?;

    // Register standalone functions
    // init_logging takes 1 arg (level)
    module.define_singleton_method("init_logging", function!(api::functions::init_logging, 1))?;
    // init_logging_from_env takes 0 args
    module.define_singleton_method(
        "init_logging_from_env",
        function!(api::functions::init_logging_from_env, 0),
    )?;
    // version takes 0 args
    module.define_singleton_method("version", function!(api::functions::version, 0))?;

    Ok(())
}
