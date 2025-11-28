//! Terminal UI utilities for progress bars and formatted output.
//!
//! This module provides a clean API for displaying build progress, status messages,
//! and formatted output in the terminal. It handles environment detection (CI, TTY)
//! and gracefully degrades when terminal features aren't available.
//!
//! # Examples
//!
//! ```no_run
//! use fob_cli::ui;
//! use std::time::Duration;
//!
//! // Initialize color support
//! ui::init_colors();
//!
//! // Show progress for multi-step operations
//! let mut progress = ui::BundleProgress::new(3);
//! let task = progress.add_task("Building modules");
//! progress.finish_task(task, "Built 5 modules");
//! progress.finish("Build complete!");
//!
//! // Simple spinner for quick tasks
//! let spinner = ui::Spinner::new("Loading config...");
//! spinner.finish("Config loaded");
//!
//! // Status messages
//! ui::success("Build successful");
//! ui::error("Failed to parse file");
//! ```

// Submodules
mod format;
mod messages;
mod progress;
mod spinner;

// Re-exports for convenient access
pub use format::{format_duration, format_size, print_build_summary};
pub use messages::{debug, error, info, success, warning};
pub use progress::BundleProgress;
pub use spinner::Spinner;

/// Check if running in a CI environment.
///
/// Detects common CI environment variables from GitHub Actions, GitLab CI,
/// CircleCI, and Travis CI.
///
/// # Returns
///
/// `true` if running in CI
pub fn is_ci() -> bool {
    std::env::var("CI").is_ok()
        || std::env::var("GITHUB_ACTIONS").is_ok()
        || std::env::var("GITLAB_CI").is_ok()
        || std::env::var("CIRCLECI").is_ok()
        || std::env::var("TRAVIS").is_ok()
}

/// Check if color output should be enabled.
///
/// Respects NO_COLOR and FORCE_COLOR environment variables, falls back to
/// terminal capability detection.
///
/// # Returns
///
/// `true` if colors should be used
pub fn should_use_color() -> bool {
    // NO_COLOR environment variable disables colors
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }

    // FORCE_COLOR enables colors even in non-TTY
    if std::env::var("FORCE_COLOR").is_ok() {
        return true;
    }

    // Check if stderr is a terminal
    console::user_attended_stderr()
}

/// Initialize color support based on environment.
///
/// Should be called early in the application lifecycle (e.g., in main).
/// Respects NO_COLOR and FORCE_COLOR environment variables.
///
/// **Note**: This function currently checks color support but doesn't modify
/// global state. The `owo-colors` crate automatically respects NO_COLOR and
/// terminal capabilities. This function is provided for explicit initialization
/// and future extensibility.
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui;
///
/// ui::init_colors();
/// // ... rest of application
/// ```
pub fn init_colors() {
    // owo-colors automatically respects NO_COLOR and terminal capabilities.
    // This function performs validation and can be extended for custom logic.
    let _ = should_use_color();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ci_no_env() {
        // Remove CI vars if they exist
        unsafe {
            std::env::remove_var("CI");
            std::env::remove_var("GITHUB_ACTIONS");
            std::env::remove_var("GITLAB_CI");
            std::env::remove_var("CIRCLECI");
            std::env::remove_var("TRAVIS");
        }

        // This might be false or true depending on test environment
        // Just verify it doesn't panic
        let _ = is_ci();
    }

    #[test]
    fn test_is_ci_with_ci_var() {
        std::env::set_var("CI", "true");
        assert!(is_ci());
        unsafe { std::env::remove_var("CI"); }
    }

    #[test]
    fn test_is_ci_with_github_actions() {
        std::env::set_var("GITHUB_ACTIONS", "true");
        assert!(is_ci());
        unsafe { std::env::remove_var("GITHUB_ACTIONS"); }
    }

    #[test]
    fn test_is_ci_with_gitlab() {
        std::env::set_var("GITLAB_CI", "true");
        assert!(is_ci());
        unsafe { std::env::remove_var("GITLAB_CI"); }
    }

    #[test]
    fn test_should_use_color_no_color() {
        std::env::set_var("NO_COLOR", "1");
        unsafe { std::env::remove_var("FORCE_COLOR"); }
        assert!(!should_use_color());
        unsafe { std::env::remove_var("NO_COLOR"); }
    }

    #[test]
    fn test_should_use_color_force_color() {
        unsafe { std::env::remove_var("NO_COLOR"); }
        std::env::set_var("FORCE_COLOR", "1");
        assert!(should_use_color());
        unsafe { std::env::remove_var("FORCE_COLOR"); }
    }

    #[test]
    fn test_should_use_color_no_color_overrides_force() {
        std::env::set_var("NO_COLOR", "1");
        std::env::set_var("FORCE_COLOR", "1");
        // NO_COLOR takes precedence
        assert!(!should_use_color());
        unsafe {
            std::env::remove_var("NO_COLOR");
            std::env::remove_var("FORCE_COLOR");
        }
    }

    #[test]
    fn test_init_colors() {
        // Should not panic
        init_colors();
    }
}
