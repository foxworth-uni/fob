//! Development server module.
//!
//! Provides a full-featured development server with:
//! - Hot reload via Server-Sent Events
//! - In-memory bundling with disk write option
//! - File watching with debouncing
//! - Error overlay in browser

pub mod asset_middleware;
pub mod builder;
pub mod config;
pub mod error_overlay;
pub mod server;
pub mod state;
pub mod watcher;

// Re-exports
pub use asset_middleware::handle_asset;
pub use builder::DevBuilder;
pub use config::DevConfig;
pub use server::DevServer;
pub use state::{BuildStatus, BundleCache, DevServerState, SharedState};
pub use watcher::{FileChange, FileWatcher};

use serde::{Deserialize, Serialize};

/// Events in the dev server lifecycle
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DevEvent {
    /// Build started
    BuildStarted,

    /// Build completed successfully
    BuildCompleted { duration_ms: u64 },

    /// Build failed with error
    BuildFailed { error: String },

    /// Client connected
    ClientConnected { id: usize },

    /// Client disconnected
    ClientDisconnected { id: usize },
}
