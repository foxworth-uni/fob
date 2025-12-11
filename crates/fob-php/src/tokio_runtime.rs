//! Global tokio runtime for PHP bindings
//!
//! PHP is single-threaded per request, so a single shared runtime
//! is sufficient and avoids the overhead of creating a new runtime
//! for each bundler operation.

use once_cell::sync::Lazy;
use tokio::runtime::Runtime;

/// Global tokio runtime shared across all bundler operations.
pub static RUNTIME: Lazy<Runtime> =
    Lazy::new(|| Runtime::new().expect("Failed to create tokio runtime"));
