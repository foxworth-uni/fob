//! Page-level browser operations and navigation.
//!
//! This module provides the Page type, which represents a browser tab/page
//! and exposes methods for navigation, script execution, and waiting.

use crate::console::{parse_console_event, ConsoleCapture};
use crate::error::{BrowserError, Result};
use crate::server::DevServer;
use crate::wait::{wait_for_result, WaitConfig};
use chromiumoxide::cdp::js_protocol::runtime::EventConsoleApiCalled;
use chromiumoxide::page::Page as ChromePage;
use futures::StreamExt;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::warn;

/// Represents a browser page (tab) with testing capabilities.
///
/// This type wraps `chromiumoxide::page::Page` and adds:
/// - Console message capture
/// - Type-safe navigation
/// - Wait helpers
/// - Resource cleanup
#[derive(Debug)]
pub struct Page {
    inner: Arc<ChromePage>,
    console: ConsoleCapture,
    _console_task: JoinHandle<()>,
}

impl Page {
    /// Creates a new Page wrapper and starts console capture.
    ///
    /// This is called internally by `TestBrowser`; users don't construct
    /// Pages directly.
    #[allow(clippy::result_large_err)]
    pub(crate) fn new(page: ChromePage) -> Self {
        let console = ConsoleCapture::new();
        let console_clone = console.clone();
        let page_arc = Arc::new(page);

        // Spawn a task to listen for console events
        let page_for_task = page_arc.clone();
        let console_task = tokio::spawn(async move {
            if let Ok(mut events) = page_for_task
                .event_listener::<EventConsoleApiCalled>()
                .await
            {
                while let Some(event) = events.next().await {
                    // In chromiumoxide 0.7.0, the event itself contains the data directly
                    let message = parse_console_event(&event);
                    console_clone.push(message);
                }
            }
        });

        Self {
            inner: page_arc,
            console,
            _console_task: console_task,
        }
    }

    /// Returns a handle to the console message capture.
    ///
    /// This allows querying accumulated console messages during or after
    /// test execution.
    #[must_use]
    pub fn console(&self) -> &ConsoleCapture {
        &self.console
    }

    /// Navigates to an absolute URL and waits for initial load.
    ///
    /// This is a low-level method. Prefer `navigate_to` for server-relative URLs.
    ///
    /// # Errors
    ///
    /// Returns `NavigationFailed` if the page fails to load or times out.
    pub async fn navigate(&self, url: &str) -> Result<()> {
        self.inner
            .goto(url)
            .await
            .map_err(|e| BrowserError::NavigationFailed {
                url: url.to_string(),
                reason: e.to_string(),
            })?;

        self.wait_for_load(WaitConfig::default()).await?;
        Ok(())
    }

    /// Navigates to a server-relative path.
    ///
    /// This is the preferred way to navigate in tests. It joins the path
    /// with the server's base URL and performs health checks.
    ///
    /// # Example
    ///
    /// ```ignore
    /// page.navigate_to(&my_server, "/dashboard").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if navigation fails or times out.
    pub async fn navigate_to(&self, server: &dyn DevServer, path: &str) -> Result<()> {
        // Health check first to fail fast
        server.health_check().await?;

        let url = server.url(path);
        self.navigate(&url).await
    }

    /// Waits for the page load event (`DOMContentLoaded`).
    ///
    /// This is automatically called by `navigate()`, but can be called
    /// manually if you trigger navigation via JavaScript.
    ///
    /// # Errors
    ///
    /// Returns an error if the wait times out or script execution fails.
    pub async fn wait_for_load(&self, config: WaitConfig) -> Result<()> {
        wait_for_result(
            || {
                let page = self.inner.clone();
                async move {
                    // Check if document.readyState is "complete"
                    let result = page
                        .evaluate("document.readyState")
                        .await
                        .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

                    let ready = result
                        .value()
                        .and_then(|v| v.as_str())
                        .is_some_and(|s| s == "complete");

                    Ok(ready)
                }
            },
            config,
            "document ready",
        )
        .await
    }

    /// Executes JavaScript in the page context and returns the result.
    ///
    /// The script runs in the main world and can access the DOM and globals.
    ///
    /// # Security
    ///
    /// Do not pass unsanitized user input to this function. Use parameterized
    /// queries via `evaluate_function()` instead.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let title: String = page.evaluate("document.title").await?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails or the result cannot be deserialized.
    pub async fn evaluate<T>(&self, script: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let result = self
            .inner
            .evaluate(script)
            .await
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

        result
            .into_value()
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))
    }

    /// Waits for a CSS selector to appear in the DOM.
    ///
    /// This repeatedly queries the page until the element exists or timeout.
    ///
    /// # Errors
    ///
    /// Returns an error if the wait times out or script execution fails.
    pub async fn wait_for_selector(&self, selector: &str, config: WaitConfig) -> Result<()> {
        let selector_owned = selector.to_string();

        wait_for_result(
            || {
                let page = self.inner.clone();
                let sel = selector_owned.clone();
                async move {
                    // Use JSON encoding for safe JavaScript string escaping
                    // This prevents injection via backticks, newlines, and other special chars
                    let escaped = serde_json::to_string(&sel)
                        .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;
                    let script = format!("!!document.querySelector({escaped})");
                    // evaluate() expects &str, not &String
                    let result = page
                        .evaluate(script.as_str())
                        .await
                        .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))?;

                    let exists = result
                        .value()
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);

                    Ok(exists)
                }
            },
            config,
            &format!("selector '{selector}'"),
        )
        .await
    }

    /// Returns the current page URL.
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails.
    pub async fn url(&self) -> Result<String> {
        self.evaluate("window.location.href").await
    }

    /// Returns the page title.
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails.
    pub async fn title(&self) -> Result<String> {
        self.evaluate("document.title").await
    }

    /// Takes a screenshot of the page and returns PNG bytes.
    ///
    /// Useful for debugging test failures in CI.
    ///
    /// # Errors
    ///
    /// Returns an error if screenshot capture fails.
    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        self.inner
            .screenshot(chromiumoxide::page::ScreenshotParams::default())
            .await
            .map_err(|e| BrowserError::ScriptExecutionFailed(e.to_string()))
    }

    /// Closes the page.
    ///
    /// This is called automatically when the Page is dropped, but can be
    /// called explicitly for cleanup.
    ///
    /// # Design Note
    ///
    /// Since our Page wraps `ChromePage` in Arc for console event sharing,
    /// we use `Arc::try_unwrap` to extract the inner page. If other Arc clones
    /// exist (e.g., the console event listener task is still running), this
    /// will fail silently and return Ok(()).
    ///
    /// # Behavior on Failure
    ///
    /// If `Arc::try_unwrap` fails (other references exist), the page is NOT closed
    /// explicitly. Instead, cleanup relies on:
    /// 1. The console event listener task completing (releases its Arc clone)
    /// 2. Chromiumoxide's Drop implementation eventually closing the page
    ///
    /// This is acceptable for a testing library, as resources will be cleaned up
    /// when the browser is closed or the test completes. For production use cases,
    /// consider implementing retry logic or a timeout-based close mechanism.
    ///
    /// # Errors
    ///
    /// Returns an error if closing the page fails.
    pub async fn close(self) -> Result<()> {
        // Try to extract the inner page from the Arc
        // This will only succeed if we're the only owner
        match Arc::try_unwrap(self.inner) {
            Ok(page) => {
                page.close().await.map_err(BrowserError::ChromiumOxide)?;
                Ok(())
            }
            Err(_arc) => {
                // Still other references exist; we can't close cleanly.
                // This is a known limitation - the page will be closed when:
                // 1. The console event listener finishes (releases Arc)
                // 2. The Browser is closed (closes all pages)
                // 3. The test completes (all resources cleaned up)
                warn!("Page::close() called but Arc has outstanding references - relying on Drop");
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // Note: Browser tests require Chrome and are in tests/integration.rs
    // These tests are for logic that doesn't require a browser

    #[test]
    fn test_selector_escaping_with_json() {
        // Test that JSON escaping prevents injection
        let test_cases = vec![
            (r#"div"#, r#""div""#),
            (r#"'injected'"#, r#""'injected'""#),
            (r#"`injected`"#, r#""`injected`""#),
        ];

        for (input, expected) in test_cases {
            let escaped = serde_json::to_string(&input).unwrap();
            assert_eq!(
                escaped, expected,
                "Selector '{}' should escape to {}",
                input, expected
            );
        }
    }

    #[test]
    fn test_json_escaping_handles_special_chars() {
        // Verify that serde_json properly escapes all dangerous characters
        let dangerous = r#"'); alert('xss');//"#;
        let escaped = serde_json::to_string(&dangerous).unwrap();

        // JSON escapes the single quote as unicode escape \u0027
        // or just leaves it as-is since it's valid in JSON strings
        // The important thing is the whole thing is wrapped in double quotes
        assert!(
            escaped.starts_with('"') && escaped.ends_with('"'),
            "Should be wrapped in double quotes"
        );

        // The dangerous string should be safely encoded within the JSON string
        // When this is used in JavaScript, it will be interpreted as a string literal
        assert!(
            escaped.len() > dangerous.len(),
            "Escaped version should include quote wrappers"
        );
    }
}
