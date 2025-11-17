//! Simple spinner for tasks without known duration.

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Simple spinner for tasks without known duration.
///
/// Useful for quick operations like loading config or checking files.
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::Spinner;
///
/// let spinner = Spinner::new("Loading config...");
/// // Do work...
/// spinner.finish("Config loaded!");
/// ```
pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    /// Create and start a new spinner.
    ///
    /// # Arguments
    ///
    /// * `message` - Initial message to display
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_cli::ui::Spinner;
    ///
    /// let spinner = Spinner::new("Loading...");
    /// // Do work...
    /// spinner.finish("Done!");
    /// ```
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .expect("valid template")
                .tick_strings(&["◐", "◓", "◑", "◒"]),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));

        Self { pb }
    }

    /// Update spinner message while it's running.
    ///
    /// # Arguments
    ///
    /// * `message` - New message to display
    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    /// Finish spinner with success message.
    ///
    /// Displays a green checkmark.
    ///
    /// # Arguments
    ///
    /// * `message` - Success message to display
    pub fn finish(&self, message: &str) {
        self.pb
            .finish_with_message(format!("{} {}", "✓".green(), message));
    }

    /// Finish spinner with error message.
    ///
    /// Displays a red X.
    ///
    /// # Arguments
    ///
    /// * `message` - Error message to display
    pub fn fail(&self, message: &str) {
        self.pb
            .finish_with_message(format!("{} {}", "✗".red(), message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        // Should not panic
        let spinner = Spinner::new("Loading...");
        spinner.set_message("Updated");
        spinner.finish("Done");
    }

    #[test]
    fn test_spinner_fail() {
        let spinner = Spinner::new("Processing");
        spinner.fail("Failed");
    }
}
