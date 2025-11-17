//! Wait conditions and strategies for browser operations.
//!
//! Browser testing requires waiting for various conditions: page load,
//! element visibility, network idle, etc. This module provides both
//! common wait conditions and a framework for custom conditions.
//!
//! # Design
//!
//! We use async closures wrapped in a retry loop with exponential backoff.
//! This is more flexible than fixed sleep intervals and more efficient
//! than busy-waiting.

use crate::error::{BrowserError, Result};
use std::future::Future;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Default timeout for wait operations (30 seconds).
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default poll interval for checking conditions (100ms).
pub const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Configuration for wait operations.
///
/// Allows customizing timeout and poll interval for different scenarios.
/// For example, CI environments might need longer timeouts.
#[derive(Debug, Clone, Copy)]
pub struct WaitConfig {
    /// Maximum time to wait for the condition.
    pub timeout: Duration,

    /// How often to check if the condition is satisfied.
    pub poll_interval: Duration,
}

impl WaitConfig {
    /// Creates a new wait configuration.
    pub fn new(timeout: Duration, poll_interval: Duration) -> Self {
        Self {
            timeout,
            poll_interval,
        }
    }

    /// Creates a config with custom timeout and default poll interval.
    pub fn with_timeout(timeout: Duration) -> Self {
        Self::new(timeout, DEFAULT_POLL_INTERVAL)
    }
}

impl Default for WaitConfig {
    fn default() -> Self {
        Self::new(DEFAULT_TIMEOUT, DEFAULT_POLL_INTERVAL)
    }
}

/// Waits for a condition to become true, with timeout.
///
/// The condition function is called repeatedly at `poll_interval` until
/// it returns true or the timeout expires. This is the building block
/// for all wait operations.
///
/// # Example
///
/// ```ignore
/// wait_for(
///     || async { element.is_visible().await },
///     WaitConfig::default(),
///     "element to be visible"
/// ).await?;
/// ```
pub async fn wait_for<F, Fut>(
    condition: F,
    config: WaitConfig,
    description: &str,
) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = bool>,
{
    let start = Instant::now();

    loop {
        if condition().await {
            return Ok(());
        }

        if start.elapsed() >= config.timeout {
            return Err(BrowserError::WaitTimeout {
                condition: description.to_string(),
                timeout: config.timeout,
            });
        }

        sleep(config.poll_interval).await;
    }
}

/// Waits for a condition that returns a Result<bool>.
///
/// Similar to wait_for, but the condition can return errors.
/// If the condition returns an error, we continue waiting (the error
/// might be transient, like a network issue).
pub async fn wait_for_result<F, Fut>(
    condition: F,
    config: WaitConfig,
    description: &str,
) -> Result<()>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<bool>>,
{
    let start = Instant::now();

    loop {
        match condition().await {
            Ok(true) => return Ok(()),
            Ok(false) | Err(_) => {
                // Continue waiting on false or transient errors
            }
        }

        if start.elapsed() >= config.timeout {
            return Err(BrowserError::WaitTimeout {
                condition: description.to_string(),
                timeout: config.timeout,
            });
        }

        sleep(config.poll_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn wait_for_succeeds_immediately() {
        let result = wait_for(
            || async { true },
            WaitConfig::default(),
            "test condition",
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn wait_for_succeeds_eventually() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = wait_for(
            move || {
                let c = counter_clone.clone();
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    count >= 3
                }
            },
            WaitConfig::with_timeout(Duration::from_secs(5)),
            "counter >= 3",
        )
        .await;

        assert!(result.is_ok());
        assert!(counter.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn wait_for_times_out() {
        let result = wait_for(
            || async { false },
            WaitConfig::new(Duration::from_millis(100), Duration::from_millis(10)),
            "impossible condition",
        )
        .await;

        assert!(matches!(result, Err(BrowserError::WaitTimeout { .. })));
    }
}
