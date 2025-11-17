//! # fob-browser-test
//!
//! A framework-agnostic browser testing library built on chromiumoxide.
//!
//! This crate provides primitives for launching headless Chrome, capturing
//! console messages, navigating pages, and waiting for conditions. It's
//! designed to be composable and reusable across different web frameworks.
//!
//! ## Architecture
//!
//! - **TestBrowser**: Manages the browser process lifecycle
//! - **Page**: Represents a browser tab with navigation and scripting
//! - **ConsoleCapture**: Thread-safe accumulation of console messages
//! - **DevServer**: Trait for integrating with dev servers
//! - **WaitConfig**: Configurable waiting strategies with timeouts
//!
//! ## Design Principles
//!
//! 1. **Framework-agnostic**: No assumptions about the web framework
//! 2. **Type-safe**: Strong types prevent common mistakes
//! 3. **Resource-safe**: Explicit Drop implementations, no leaked processes
//! 4. **Async-first**: Built on tokio for efficient I/O
//! 5. **Testable**: Easy to use in cargo tests with minimal boilerplate
//!
//! ## Example Usage
//!
//! ```ignore
//! use fob_browser_test::{TestBrowser, TestBrowserConfig};
//!
//! #[tokio::test]
//! async fn test_page_navigation() -> Result<(), Box<dyn std::error::Error>> {
//!     // Launch browser
//!     let browser = TestBrowser::launch(TestBrowserConfig::default()).await?;
//!
//!     // Create a page
//!     let page = browser.new_page().await?;
//!
//!     // Navigate and capture console
//!     page.navigate("http://localhost:3000").await?;
//!
//!     // Check for errors
//!     let console = page.console();
//!     assert_eq!(console.error_count(), 0, "No console errors");
//!
//!     // Verify page state
//!     let title: String = page.evaluate("document.title").await?;
//!     assert_eq!(title, "My App");
//!
//!     // Cleanup
//!     browser.close().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Integration with Framework-Specific Harnesses
//!
//! Framework-specific crates (like gumbo-test-harness) build on this library:
//!
//! ```ignore
//! // In gumbo-test-harness
//! pub struct GumboTestHarness {
//!     browser: TestBrowser,
//!     server: GumboDevServer,
//! }
//!
//! impl GumboTestHarness {
//!     pub async fn new() -> Result<Self> {
//!         let server = GumboDevServer::start().await?;
//!         let browser = TestBrowser::launch(TestBrowserConfig::default()).await?;
//!         Ok(Self { browser, server })
//!     }
//!
//!     pub async fn navigate(&self, path: &str) -> Result<Page> {
//!         let page = self.browser.new_page().await?;
//!         page.navigate_to(&self.server, path).await?;
//!         Ok(page)
//!     }
//! }
//! ```
//!
//! ## Security Considerations
//!
//! - **Input validation**: All URLs and scripts are validated before execution
//! - **No unsafe code**: Relies on chromiumoxide's safety guarantees
//! - **Process isolation**: Each browser instance runs in a separate process
//! - **Cleanup guarantees**: Drop ensures processes don't leak
//!
//! ## Performance
//!
//! - Browser launch: ~500ms (one-time cost)
//! - Page creation: ~50ms per page
//! - Navigation: Depends on page complexity
//! - Console capture: Zero-cost until queried
//!
//! ## Testing Strategy
//!
//! This crate uses two levels of testing:
//!
//! 1. **Unit tests**: Mock-free logic tests (console filtering, wait strategies)
//! 2. **Integration tests**: Real browser tests (require Chrome installed)
//!
//! Run with `cargo test` (unit) or `cargo test --ignored` (integration).

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod browser;
pub mod console;
pub mod error;
pub mod page;
pub mod server;
pub mod wait;

// Re-export main types for convenience
pub use browser::{TestBrowser, TestBrowserConfig};
pub use console::{ConsoleCapture, ConsoleLevel, ConsoleMessage};
pub use error::{BrowserError, Result};
pub use page::Page;
pub use server::{DevServer, StaticUrlServer};
pub use wait::{WaitConfig, DEFAULT_POLL_INTERVAL, DEFAULT_TIMEOUT};
