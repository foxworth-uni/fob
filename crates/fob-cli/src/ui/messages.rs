//! Status message functions for terminal output.

use owo_colors::OwoColorize;

/// Print a success message to stderr.
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::success;
///
/// success("Build completed successfully");
/// ```
pub fn success(message: &str) {
    eprintln!("{} {}", "✓".green().bold(), message);
}

/// Print an info message to stderr.
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::info;
///
/// info("Starting build process...");
/// ```
pub fn info(message: &str) {
    eprintln!("{} {}", "ℹ".blue().bold(), message);
}

/// Print a warning message to stderr.
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::warning;
///
/// warning("No TypeScript files found for .d.ts generation");
/// ```
pub fn warning(message: &str) {
    eprintln!("{} {}", "⚠".yellow().bold(), message.yellow());
}

/// Print an error message to stderr.
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::error;
///
/// error("Failed to read configuration file");
/// ```
pub fn error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Print a debug message to stderr (only if RUST_LOG is set).
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::debug;
///
/// debug("Resolved module path: /path/to/module");
/// ```
pub fn debug(message: &str) {
    if std::env::var("RUST_LOG").is_ok() {
        eprintln!("{} {}", "◆".dimmed(), message.dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_messages() {
        // These should not panic
        success("Success message");
        info("Info message");
        warning("Warning message");
        error("Error message");
        debug("Debug message");
    }
}
