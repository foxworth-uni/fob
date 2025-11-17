//! Formatting utilities for sizes, durations, and build summaries.

use console::Term;
use owo_colors::OwoColorize;
use std::time::Duration;

/// Format file size in human-readable format.
///
/// Converts bytes to the most appropriate unit (B, KB, MB, GB).
///
/// # Arguments
///
/// * `bytes` - Size in bytes
///
/// # Returns
///
/// Formatted string (e.g., "1.50 MB")
///
/// # Examples
///
/// ```
/// use fob_cli::ui::format_size;
///
/// assert_eq!(format_size(0), "0 B");
/// assert_eq!(format_size(500), "500 B");
/// assert_eq!(format_size(1024), "1.00 KB");
/// assert_eq!(format_size(1_048_576), "1.00 MB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    if unit_idx == 0 {
        format!("{} {}", size as u64, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size, UNITS[unit_idx])
    }
}

/// Format duration in human-readable format.
///
/// Converts to the most appropriate unit (ms, s, m:s).
///
/// # Arguments
///
/// * `duration` - Duration to format
///
/// # Returns
///
/// Formatted string (e.g., "1.50s", "2m 30s")
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use fob_cli::ui::format_duration;
///
/// assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
/// assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
/// assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
/// ```
pub fn format_duration(duration: Duration) -> String {
    let total_ms = duration.as_millis();

    if total_ms < 1000 {
        format!("{}ms", total_ms)
    } else if total_ms < 60_000 {
        format!("{:.2}s", duration.as_secs_f64())
    } else {
        let secs = duration.as_secs();
        let mins = secs / 60;
        let secs = secs % 60;
        format!("{}m {}s", mins, secs)
    }
}

/// Print a build summary table to stderr.
///
/// Displays a formatted table of build outputs with sizes and durations,
/// plus a total summary.
///
/// # Arguments
///
/// * `entries` - Slice of (name, size_bytes, duration) tuples
///
/// # Examples
///
/// ```no_run
/// use std::time::Duration;
/// use fob_cli::ui::print_build_summary;
///
/// print_build_summary(&[
///     ("index.js".to_string(), 15_234, Duration::from_millis(450)),
///     ("vendor.js".to_string(), 234_567, Duration::from_millis(1200)),
/// ]);
/// ```
pub fn print_build_summary(entries: &[(String, u64, Duration)]) {
    let term = Term::stderr();
    let width = term.size().1 as usize;

    // Header
    eprintln!("\n{}", "Build Summary".bold().underline());
    eprintln!("{}", "─".repeat(width.min(80)));

    // Table entries
    for (name, size, duration) in entries {
        let size_str = format_size(*size);
        let dur_str = format_duration(*duration);

        eprintln!(
            "  {} {} {} {}",
            "▸".blue(),
            name.bright_white().bold(),
            size_str.dimmed(),
            format!("({})", dur_str).dimmed()
        );
    }

    // Footer
    eprintln!("{}", "─".repeat(width.min(80)));

    let total_size: u64 = entries.iter().map(|(_, s, _)| s).sum();
    let total_time: Duration = entries.iter().map(|(_, _, d)| d).sum();

    eprintln!(
        "  {} {} in {}",
        "Total:".bold(),
        format_size(total_size).green(),
        format_duration(total_time).green()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_zero() {
        assert_eq!(format_size(0), "0 B");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(10_240), "10.00 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1_048_576), "1.00 MB");
        assert_eq!(format_size(1_572_864), "1.50 MB");
        assert_eq!(format_size(10_485_760), "10.00 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1_073_741_824), "1.00 GB");
        assert_eq!(format_size(2_147_483_648), "2.00 GB");
    }

    #[test]
    fn test_format_duration_milliseconds() {
        assert_eq!(format_duration(Duration::from_millis(0)), "0ms");
        assert_eq!(format_duration(Duration::from_millis(50)), "50ms");
        assert_eq!(format_duration(Duration::from_millis(999)), "999ms");
    }

    #[test]
    fn test_format_duration_seconds() {
        assert_eq!(format_duration(Duration::from_millis(1000)), "1.00s");
        assert_eq!(format_duration(Duration::from_millis(1500)), "1.50s");
        assert_eq!(format_duration(Duration::from_millis(59_999)), "60.00s");
    }

    #[test]
    fn test_format_duration_minutes() {
        assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "61m 1s");
    }

    #[test]
    fn test_print_build_summary() {
        let entries = vec![
            ("index.js".to_string(), 15_234, Duration::from_millis(450)),
            (
                "vendor.js".to_string(),
                234_567,
                Duration::from_millis(1200),
            ),
        ];

        // Should not panic
        print_build_summary(&entries);
    }

    #[test]
    fn test_print_build_summary_empty() {
        // Should handle empty input gracefully
        print_build_summary(&[]);
    }
}
