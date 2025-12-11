//! Python bindings for Fob bundler core
//!
//! This module provides PyO3 bindings that mirror the Node.js API,
//! allowing Python users to bundle JavaScript/TypeScript code using Fob.

mod api;
mod conversion;
mod core;
mod error;
mod runtime;
mod types;

use pyo3::prelude::*;

/// Python module initialization
#[pymodule]
fn fob(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Register error types
    error::register_errors(m)?;

    // Add FobError to module
    m.add("FobError", m.py().get_type::<error::FobError>())?;

    // Register types
    types::register_types(m)?;

    // Register config types
    api::config::register_config_types(m)?;

    // Register primitives
    api::primitives::register_primitives(m)?;

    // Register Fob class
    m.add_class::<api::bundler::Fob>()?;

    // Register standalone functions
    m.add_function(wrap_pyfunction!(api::functions::init_logging, m)?)?;
    m.add_function(wrap_pyfunction!(api::functions::init_logging_from_env, m)?)?;
    m.add_function(wrap_pyfunction!(api::functions::bundle_single, m)?)?;
    m.add_function(wrap_pyfunction!(api::functions::version, m)?)?;

    Ok(())
}
