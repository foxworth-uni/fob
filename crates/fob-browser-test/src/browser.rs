//! Browser lifecycle management and process control.
//!
//! This module provides `TestBrowser`, the main entry point for browser testing.
//! It handles launching Chrome, managing the process lifecycle, and creating
//! pages for navigation.
//!
//! # Resource Safety
//!
//! `TestBrowser` implements Drop to ensure the browser process is killed even
//! if tests panic. However, explicit cleanup via `close()` is preferred for
//! graceful shutdown.

use crate::error::{BrowserError, Result};
use crate::page::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Configuration for launching a test browser.
///
/// Provides sensible defaults for headless testing with options
/// to customize for debugging or CI environments.
#[derive(Debug, Clone)]
pub struct TestBrowserConfig {
    /// Run in headless mode (default: true).
    pub headless: bool,

    /// Browser window size (default: 1920x1080).
    pub window_size: (u32, u32),

    /// Additional Chrome arguments.
    pub args: Vec<String>,

    /// Chrome executable path (None = auto-detect).
    pub chrome_path: Option<String>,
}

impl TestBrowserConfig {
    /// Creates a new config with defaults for headless testing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enables visible mode for debugging.
    ///
    /// When headless is false, you can watch the browser execute tests.
    #[must_use]
    pub fn visible(mut self) -> Self {
        self.headless = false;
        self
    }

    /// Sets a custom window size.
    #[must_use]
    pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_size = (width, height);
        self
    }

    /// Adds additional Chrome arguments.
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args.extend(args);
        self
    }

    /// Converts to chromiumoxide `BrowserConfig`.
    #[allow(clippy::result_large_err)]
    fn to_browser_config(&self) -> Result<BrowserConfig> {
        let mut config = BrowserConfig::builder();

        if self.headless {
            config = config.arg("--headless");
        }

        config = config.arg(format!(
            "--window-size={},{}",
            self.window_size.0, self.window_size.1
        ));

        // Create a unique user data directory to avoid ProcessSingleton conflicts
        // when running multiple browser instances in parallel (e.g., during tests)
        // Using UUID v4 ensures uniqueness without risk of TOCTOU race conditions
        let temp_dir = std::env::temp_dir();
        let unique_id = uuid::Uuid::new_v4();
        let user_data_dir = temp_dir.join(format!("fob-browser-test-{unique_id}"));
        config = config.arg(format!("--user-data-dir={}", user_data_dir.display()));

        for arg in &self.args {
            config = config.arg(arg.clone());
        }

        if let Some(path) = &self.chrome_path {
            config = config.chrome_executable(path.clone());
        }

        config.build().map_err(|e| BrowserError::LaunchFailed {
            reason: format!("invalid browser configuration: {e}"),
            source: None,
        })
    }
}

impl Default for TestBrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            window_size: (1920, 1080),
            args: vec![
                // Security Note: --no-sandbox disables Chrome's security sandbox
                // This is SAFE for isolated test environments (CI/Docker) but should
                // NEVER be used with untrusted content in production.
                // Required when user namespaces are unavailable (common in containers).
                "--no-sandbox".to_string(),
                // Prevents /dev/shm exhaustion in containerized environments
                "--disable-dev-shm-usage".to_string(),
            ],
            chrome_path: None,
        }
    }
}

/// A managed browser instance for testing.
///
/// This is the main entry point for browser testing. It wraps the browser
/// process, handles lifecycle, and provides methods to create pages.
///
/// # Example
///
/// ```ignore
/// let browser = TestBrowser::launch(TestBrowserConfig::default()).await?;
/// let page = browser.new_page().await?;
/// page.navigate("https://example.com").await?;
/// // Tests run...
/// browser.close().await?;
/// ```
///
/// # Resource Management
///
/// `TestBrowser` implements Drop to kill the browser process if not explicitly
/// closed. However, relying on Drop is not ideal because:
/// 1. Drop is synchronous; we can't await the close operation
/// 2. Panics in Drop are usually hidden
/// 3. Explicit cleanup is more testable
///
/// Prefer calling `close()` explicitly at the end of tests.
pub struct TestBrowser {
    inner: Arc<Mutex<Option<Browser>>>,
}

impl TestBrowser {
    /// Launches a new browser instance with the given configuration.
    ///
    /// This spawns a Chrome process and establishes a CDP connection.
    ///
    /// # Errors
    ///
    /// Returns `LaunchFailed` if Chrome is not installed, not executable,
    /// or fails to start.
    pub async fn launch(config: TestBrowserConfig) -> Result<Self> {
        debug!("Launching browser with config: {:?}", config);

        let browser_config = config.to_browser_config()?;

        let (browser, mut handler) =
            Browser::launch(browser_config)
                .await
                .map_err(|e| BrowserError::LaunchFailed {
                    reason: "failed to launch Chrome process".to_string(),
                    source: Some(Box::new(e)),
                })?;

        // Spawn a task to drive the browser handler
        // This is required for chromiumoxide to process CDP events
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    warn!("Browser handler error: {}", e);
                }
            }
        });

        debug!("Browser launched successfully");

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(browser))),
        })
    }

    /// Creates a new browser page (tab).
    ///
    /// Each page has independent state and console capture.
    ///
    /// # Errors
    ///
    /// Returns `AlreadyClosed` if the browser has been closed.
    pub async fn new_page(&self) -> Result<Page> {
        let browser = self.inner.lock().await;

        let browser = browser.as_ref().ok_or(BrowserError::AlreadyClosed)?;

        let chrome_page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;

        Ok(Page::new(chrome_page))
    }

    /// Closes the browser and kills the Chrome process.
    ///
    /// This should be called explicitly at the end of tests for graceful
    /// shutdown. If not called, Drop will kill the process forcefully.
    ///
    /// # Errors
    ///
    /// Returns an error if the browser fails to close gracefully.
    pub async fn close(self) -> Result<()> {
        let mut browser_guard = self.inner.lock().await;

        if let Some(mut browser) = browser_guard.take() {
            debug!("Closing browser gracefully");
            // Browser::close() requires &mut self
            browser
                .close()
                .await
                .map_err(|e| BrowserError::ConnectionFailed(e.to_string()))?;
        }

        Ok(())
    }

    /// Returns true if the browser has been closed.
    pub async fn is_closed(&self) -> bool {
        self.inner.lock().await.is_none()
    }
}

impl Drop for TestBrowser {
    fn drop(&mut self) {
        // We can't call async methods in Drop, so we rely on chromiumoxide's
        // Browser Drop implementation to kill the Chrome process.
        //
        // Behavior: When this TestBrowser is dropped, the inner Arc<Mutex<Option<Browser>>>
        // is dropped. If the Browser hasn't been taken out (via close()), chromiumoxide's
        // Browser::drop() will forcefully terminate the Chrome process.
        //
        // This ensures no leaked processes even if tests panic before calling close().
        // However, explicit close() is preferred for graceful shutdown.
        warn!("TestBrowser dropped without explicit close() - forcing shutdown via Drop");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Chrome to be installed
    async fn browser_launch_and_close() {
        let browser = TestBrowser::launch(TestBrowserConfig::default())
            .await
            .expect("failed to launch browser");

        assert!(!browser.is_closed().await);

        browser.close().await.expect("failed to close browser");
    }

    #[tokio::test]
    #[ignore]
    async fn browser_create_page() {
        let browser = TestBrowser::launch(TestBrowserConfig::default())
            .await
            .expect("failed to launch");

        let page = browser.new_page().await.expect("failed to create page");

        // Verify we can navigate
        page.navigate("about:blank")
            .await
            .expect("failed to navigate");

        browser.close().await.expect("failed to close");
    }
}
