//! Error types for browser testing operations.
//!
//! This module provides a structured error hierarchy that distinguishes between
//! different failure modes: browser launch failures, navigation errors, timeout
//! issues, and console-related problems. Each error type includes context to aid
//! debugging.

use std::time::Duration;
use thiserror::Error;

/// The main error type for all browser testing operations.
///
/// This enum uses thiserror to provide both Display implementations and
/// error source chaining. Each variant includes relevant context about
/// what operation failed and why.
#[derive(Debug, Error)]
pub enum BrowserError {
    /// Failed to launch the browser process.
    ///
    /// This typically occurs when Chrome/Chromium is not installed,
    /// or when there are permission issues with the executable.
    #[error("failed to launch browser: {reason}")]
    LaunchFailed {
        /// Human-readable reason for the launch failure
        reason: String,
        /// Optional underlying error that caused the failure
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Failed to establish Chrome DevTools Protocol connection.
    ///
    /// This can happen if the browser crashes immediately after launch
    /// or if the WebSocket connection is rejected.
    #[error("CDP connection failed: {0}")]
    ConnectionFailed(String),

    /// Navigation to a URL failed or timed out.
    ///
    /// Includes the URL that failed and the underlying reason.
    #[error("navigation to '{url}' failed: {reason}")]
    NavigationFailed {
        /// The URL that failed to load
        url: String,
        /// Reason for the navigation failure
        reason: String
    },

    /// A wait condition was not satisfied within the timeout.
    ///
    /// This is used for operations like wait_for_load, wait_for_selector, etc.
    #[error("wait condition '{condition}' timed out after {timeout:?}")]
    WaitTimeout {
        /// Description of the condition that timed out
        condition: String,
        /// How long we waited before timing out
        timeout: Duration,
    },

    /// JavaScript execution in the page context failed.
    ///
    /// Includes the script snippet (truncated) and the error message.
    #[error("JavaScript execution failed: {0}")]
    ScriptExecutionFailed(String),

    /// The browser process crashed or was killed unexpectedly.
    #[error("browser process terminated unexpectedly")]
    ProcessTerminated,

    /// An operation was attempted on a closed browser instance.
    #[error("browser instance is already closed")]
    AlreadyClosed,

    /// Wraps errors from the chromiumoxide library.
    #[error("chromiumoxide error: {0}")]
    ChromiumOxide(#[from] chromiumoxide::error::CdpError),

    /// Generic I/O errors (file access, network, etc.).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// A specialized Result type for browser operations.
pub type Result<T> = std::result::Result<T, BrowserError>;
