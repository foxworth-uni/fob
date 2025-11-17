pub mod bundle;
pub mod config;
pub mod dev;
pub mod discovery;
pub mod error;
pub mod settings;
pub mod validation;

#[cfg(feature = "eval")]
pub mod eval;

// Re-export main types
pub use bundle::*;
pub use config::*;
pub use dev::*;
pub use error::*;
pub use settings::*;

// Re-export discovery and validation
pub use discovery::{discover, discover_with_profile, ConfigDiscovery};
pub use validation::{validate_fs, validate_schema, ConfigValidator, FsValidator, SchemaValidator};
