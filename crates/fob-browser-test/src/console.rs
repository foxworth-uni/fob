//! Console message capture and filtering.
//!
//! This module provides strongly-typed console messages that preserve the
//! severity level, timestamp, and source location. The `ConsoleCapture` type
//! accumulates messages during test execution and provides filtering/querying.
//!
//! # Design Rationale
//!
//! We use Arc<Mutex<Vec<ConsoleMessage>>> instead of channels because:
//! 1. Tests need to query accumulated messages multiple times
//! 2. Message ordering must be preserved
//! 3. No backpressure concerns (test workloads are small)
//! 4. Simpler API - no need to drain channels

use chromiumoxide::cdp::js_protocol::runtime::EventConsoleApiCalled;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// The severity level of a console message.
///
/// Maps directly to JavaScript console methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsoleLevel {
    /// `console.log()`
    Log,
    /// `console.info()`
    Info,
    /// `console.warn()`
    Warning,
    /// `console.error()`
    Error,
    /// `console.debug()`
    Debug,
    /// Catch-all for other console APIs
    Other,
}

impl ConsoleLevel {
    /// Returns true if this is an error-level message.
    ///
    /// Useful for assertions: `assert!(console.has_errors() == false)`
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, ConsoleLevel::Error)
    }

    /// Returns true if this is a warning or error.
    #[must_use]
    pub fn is_warning_or_error(&self) -> bool {
        matches!(self, ConsoleLevel::Warning | ConsoleLevel::Error)
    }
}

impl From<&str> for ConsoleLevel {
    fn from(s: &str) -> Self {
        match s {
            "log" => ConsoleLevel::Log,
            "info" => ConsoleLevel::Info,
            "warning" => ConsoleLevel::Warning,
            "error" => ConsoleLevel::Error,
            "debug" => ConsoleLevel::Debug,
            _ => ConsoleLevel::Other,
        }
    }
}

impl From<&EventConsoleApiCalled> for ConsoleLevel {
    /// Converts from chromiumoxide's `ConsoleApiCalledType` to our `ConsoleLevel`.
    ///
    /// # Design Note
    ///
    /// The `chromiumoxide_cdp` types are serde-generated from the Chrome `DevTools`
    /// Protocol PDL files. `ConsoleApiCalledType` is an enum, but the exact set of
    /// conversion methods varies by version. We pattern match directly on the variants
    /// to avoid depending on unstable API surface.
    fn from(event: &EventConsoleApiCalled) -> Self {
        use chromiumoxide::cdp::js_protocol::runtime::ConsoleApiCalledType;

        match event.r#type {
            ConsoleApiCalledType::Log => ConsoleLevel::Log,
            ConsoleApiCalledType::Info => ConsoleLevel::Info,
            ConsoleApiCalledType::Warning => ConsoleLevel::Warning,
            ConsoleApiCalledType::Error => ConsoleLevel::Error,
            ConsoleApiCalledType::Debug => ConsoleLevel::Debug,
            _ => ConsoleLevel::Other,
        }
    }
}

/// A captured console message with metadata.
///
/// Includes the severity level, formatted message text, timestamp,
/// and optional source location (`<file:line:column>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleMessage {
    /// Severity level (log, warn, error, etc.)
    pub level: ConsoleLevel,

    /// The formatted message text. Multiple arguments are joined with spaces.
    pub text: String,

    /// When the message was captured (system time, not page time).
    pub timestamp: SystemTime,

    /// Source location if available (e.g., "app.js:42:10").
    pub source: Option<String>,
}

impl ConsoleMessage {
    /// Creates a new console message.
    #[must_use]
    pub fn new(level: ConsoleLevel, text: String) -> Self {
        Self {
            level,
            text,
            timestamp: SystemTime::now(),
            source: None,
        }
    }

    /// Creates a message with source location.
    #[must_use]
    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }
}

/// Thread-safe console message accumulator.
///
/// This type is cheaply cloneable (Arc) and allows concurrent access
/// from the CDP event handler and test code. Messages are accumulated
/// in arrival order and can be filtered by level.
///
/// # Example
///
/// ```ignore
/// let capture = ConsoleCapture::new();
/// // Messages are added by the browser's CDP event handler
/// // Later in tests:
/// assert_eq!(capture.error_count(), 0, "No console errors");
/// ```
#[derive(Debug, Clone)]
pub struct ConsoleCapture {
    messages: Arc<Mutex<Vec<ConsoleMessage>>>,
}

impl ConsoleCapture {
    /// Creates a new, empty console capture.
    #[must_use]
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a message to the capture buffer.
    ///
    /// Called by the CDP event handler when Runtime.consoleAPICalled fires.
    /// This method is thread-safe and non-blocking.
    ///
    /// # Behavior on Mutex Poisoning
    ///
    /// If the internal mutex is poisoned (a panic occurred while holding the lock),
    /// the message is **silently dropped**. This is acceptable for a testing library
    /// because:
    /// 1. Poisoning indicates a serious test failure has already occurred
    /// 2. The test will fail anyway due to the panic
    /// 3. Missing console messages are less critical than propagating the panic
    ///
    /// This silent failure means some console output may be lost if tests panic while
    /// capturing console messages, but the primary test failure will still be visible.
    pub(crate) fn push(&self, message: ConsoleMessage) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.push(message);
        }
        // Note: If lock fails (mutex poisoned), message is silently dropped.
        // This can occur if a panic happened while holding the lock.
        // Acceptable for test tooling - the panic is the primary concern.
    }

    /// Returns all captured messages as a snapshot.
    ///
    /// This clones the message vector to avoid holding the lock.
    /// For large test suites, consider using iterators instead.
    #[must_use]
    pub fn messages(&self) -> Vec<ConsoleMessage> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Returns messages filtered by level.
    #[must_use]
    pub fn messages_with_level(&self, level: ConsoleLevel) -> Vec<ConsoleMessage> {
        self.messages()
            .into_iter()
            .filter(|m| m.level == level)
            .collect()
    }

    /// Returns all error-level messages.
    #[must_use]
    pub fn errors(&self) -> Vec<ConsoleMessage> {
        self.messages_with_level(ConsoleLevel::Error)
    }

    /// Returns all warning-level messages.
    #[must_use]
    pub fn warnings(&self) -> Vec<ConsoleMessage> {
        self.messages_with_level(ConsoleLevel::Warning)
    }

    /// Returns the count of error messages.
    ///
    /// More efficient than `errors().len()` as it doesn't clone.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|m| m.level.is_error())
            .count()
    }

    /// Returns the count of warning or error messages.
    #[must_use]
    pub fn warning_or_error_count(&self) -> usize {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter(|m| m.level.is_warning_or_error())
            .count()
    }

    /// Returns true if any error messages were captured.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Clears all captured messages.
    ///
    /// Useful when reusing a browser instance across multiple test cases.
    pub fn clear(&self) {
        if let Ok(mut messages) = self.messages.lock() {
            messages.clear();
        }
    }

    /// Returns the total number of messages captured.
    #[must_use]
    pub fn len(&self) -> usize {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Returns true if no messages have been captured.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for ConsoleCapture {
    fn default() -> Self {
        Self::new()
    }
}

/// Parses a CDP `EventConsoleApiCalled` into a `ConsoleMessage`.
///
/// This function handles the conversion from chromiumoxide's protocol types
/// to our domain types. Arguments are formatted and joined with spaces.
///
/// # Design Note
///
/// In chromiumoxide 0.7.0, `EventConsoleApiCalled` contains the event fields directly
/// as public members, rather than wrapping them in a separate `params` field.
/// The event structure includes: type, args, `execution_context_id`, timestamp, etc.
pub(crate) fn parse_console_event(event: &EventConsoleApiCalled) -> ConsoleMessage {
    let level = ConsoleLevel::from(event);

    // Format arguments - each arg can be a primitive or object
    let text = event
        .args
        .iter()
        .map(|arg| {
            arg.value
                .as_ref()
                .and_then(|v| v.as_str())
                .unwrap_or("<object>")
                .to_string()
        })
        .collect::<Vec<_>>()
        .join(" ");

    let mut message = ConsoleMessage::new(level, text);

    // Add source location if available
    if let Some(stack_trace) = &event.stack_trace {
        if let Some(frame) = stack_trace.call_frames.first() {
            let source = format!(
                "{}:{}:{}",
                frame.url, frame.line_number, frame.column_number
            );
            message = message.with_source(source);
        }
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn console_level_classification() {
        assert!(ConsoleLevel::Error.is_error());
        assert!(!ConsoleLevel::Warning.is_error());
        assert!(ConsoleLevel::Error.is_warning_or_error());
        assert!(ConsoleLevel::Warning.is_warning_or_error());
        assert!(!ConsoleLevel::Log.is_warning_or_error());
    }

    #[test]
    fn console_capture_accumulation() {
        let capture = ConsoleCapture::new();

        capture.push(ConsoleMessage::new(ConsoleLevel::Log, "info".into()));
        capture.push(ConsoleMessage::new(ConsoleLevel::Error, "bad".into()));
        capture.push(ConsoleMessage::new(ConsoleLevel::Warning, "warn".into()));

        assert_eq!(capture.len(), 3);
        assert_eq!(capture.error_count(), 1);
        assert_eq!(capture.warning_or_error_count(), 2);
        assert!(capture.has_errors());
    }

    #[test]
    fn console_capture_filtering() {
        let capture = ConsoleCapture::new();

        capture.push(ConsoleMessage::new(ConsoleLevel::Log, "log1".into()));
        capture.push(ConsoleMessage::new(ConsoleLevel::Error, "err1".into()));
        capture.push(ConsoleMessage::new(ConsoleLevel::Log, "log2".into()));

        let errors = capture.errors();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].text, "err1");

        let logs = capture.messages_with_level(ConsoleLevel::Log);
        assert_eq!(logs.len(), 2);
    }

    #[test]
    fn console_capture_clear() {
        let capture = ConsoleCapture::new();
        capture.push(ConsoleMessage::new(ConsoleLevel::Log, "test".into()));
        assert_eq!(capture.len(), 1);

        capture.clear();
        assert_eq!(capture.len(), 0);
        assert!(capture.is_empty());
    }
}
