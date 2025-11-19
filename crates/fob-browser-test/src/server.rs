//! Dev server lifecycle management abstraction.
//!
//! This module provides the `DevServer` trait that framework-specific test
//! harnesses implement. fob-browser-test doesn't know how to start servers,
//! but it can consume URLs from anything that implements this trait.
//!
//! # Design Philosophy
//!
//! The trait is intentionally minimal - it only provides URL access, not
//! lifecycle management. Higher-level crates (gumbo-test-harness) handle
//! starting/stopping servers and implement this trait.

use crate::error::Result;
use async_trait::async_trait;
use std::fmt;

/// Represents a running development server.
///
/// Framework-specific test harnesses implement this trait to provide
/// a uniform interface for browser tests. The trait is object-safe,
/// allowing dynamic dispatch when needed.
///
/// # Example Implementation
///
/// ```ignore
/// struct GumboDevServer {
///     base_url: String,
///     _handle: ServerHandle,
/// }
///
/// #[async_trait]
/// impl DevServer for GumboDevServer {
///     fn base_url(&self) -> &str {
///         &self.base_url
///     }
///
///     async fn health_check(&self) -> Result<()> {
///         // Ping /health endpoint
///         Ok(())
///     }
/// }
/// ```
#[async_trait]
pub trait DevServer: Send + Sync {
    /// Returns the base URL of the server (e.g., `<http://localhost:3000>`).
    ///
    /// This URL should NOT include a trailing slash. Paths will be joined
    /// using `url::Url::join()` semantics.
    fn base_url(&self) -> &str;

    /// Performs a health check to ensure the server is responsive.
    ///
    /// This is called before navigation to fail fast if the server is down.
    /// The default implementation returns Ok(()), assuming the server is healthy.
    async fn health_check(&self) -> Result<()> {
        Ok(())
    }

    /// Returns a full URL by joining a path to the base URL.
    ///
    /// # Example
    ///
    /// ```ignore
    /// server.url("/app/dashboard") // "http://localhost:3000/app/dashboard"
    /// ```
    fn url(&self, path: &str) -> String {
        let base = self.base_url().trim_end_matches('/');
        let path = path.trim_start_matches('/');
        format!("{base}/{path}")
    }
}

impl fmt::Debug for dyn DevServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DevServer")
            .field("base_url", &self.base_url())
            .finish()
    }
}

/// A simple static URL "server" for testing against external URLs.
///
/// This implementation is useful when you want to test against a URL
/// that's already running (not managed by the test harness).
///
/// # Example
///
/// ```ignore
/// let server = StaticUrlServer::new("http://localhost:8080");
/// browser.navigate_to(&server, "/").await?;
/// ```
#[derive(Debug, Clone)]
pub struct StaticUrlServer {
    base_url: String,
}

impl StaticUrlServer {
    /// Creates a new static URL server.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

#[async_trait]
impl DevServer for StaticUrlServer {
    fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_server_url_joining() {
        let server = StaticUrlServer::new("http://localhost:3000");
        assert_eq!(server.url("/app"), "http://localhost:3000/app");
        assert_eq!(server.url("app"), "http://localhost:3000/app");

        let server_with_slash = StaticUrlServer::new("http://localhost:3000/");
        assert_eq!(server_with_slash.url("/app"), "http://localhost:3000/app");
    }
}
