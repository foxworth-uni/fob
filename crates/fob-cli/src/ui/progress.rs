//! Progress tracking for multi-step bundling operations.

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use std::time::Duration;

/// Progress tracker for bundling operations.
///
/// Provides a main progress bar and multiple subtask spinners for detailed
/// feedback during multi-step build operations. Automatically cleans up on drop.
///
/// # Examples
///
/// ```no_run
/// use fob_cli::ui::BundleProgress;
///
/// let mut progress = BundleProgress::new(5);
/// let task1 = progress.add_task("Parsing modules");
/// progress.finish_task(task1, "Parsed 50 modules");
///
/// let task2 = progress.add_task("Transforming code");
/// progress.finish_task(task2, "Transformed TypeScript");
///
/// progress.finish("Build complete!");
/// ```
pub struct BundleProgress {
    multi: MultiProgress,
    main_bar: ProgressBar,
    task_bars: Vec<ProgressBar>,
}

impl BundleProgress {
    /// Create a new progress tracker with the specified number of total tasks.
    ///
    /// # Arguments
    ///
    /// * `total_tasks` - Total number of tasks to complete
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use fob_cli::ui::BundleProgress;
    ///
    /// let mut progress = BundleProgress::new(5);
    /// // Add and complete tasks...
    /// ```
    pub fn new(total_tasks: u64) -> Self {
        let multi = MultiProgress::new();

        let main_bar = multi.add(ProgressBar::new(total_tasks));
        main_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .expect("valid template")
                .progress_chars("█▓▒░"),
        );
        main_bar.enable_steady_tick(Duration::from_millis(100));

        Self {
            multi,
            main_bar,
            task_bars: Vec::new(),
        }
    }

    /// Add a subtask progress spinner.
    ///
    /// Returns the task ID which can be used to update or finish the task.
    ///
    /// # Arguments
    ///
    /// * `name` - Initial message to display for this task
    ///
    /// # Returns
    ///
    /// Task ID for updating this task's status
    pub fn add_task(&mut self, name: &str) -> usize {
        let pb = self.multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("  {spinner:.blue} {msg}")
                .expect("valid template")
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.set_message(format!("{}", name.dimmed()));
        pb.enable_steady_tick(Duration::from_millis(80));

        let idx = self.task_bars.len();
        self.task_bars.push(pb);
        idx
    }

    /// Update a task's status message.
    ///
    /// # Arguments
    ///
    /// * `task_id` - ID returned from `add_task`
    /// * `message` - New message to display
    pub fn update_task(&self, task_id: usize, message: &str) {
        if let Some(pb) = self.task_bars.get(task_id) {
            pb.set_message(message.to_string());
        }
    }

    /// Mark a task as successfully completed.
    ///
    /// Increments the main progress bar and displays a success checkmark.
    ///
    /// # Arguments
    ///
    /// * `task_id` - ID returned from `add_task`
    /// * `message` - Completion message to display
    pub fn finish_task(&mut self, task_id: usize, message: &str) {
        if let Some(pb) = self.task_bars.get(task_id) {
            pb.finish_with_message(format!("  {} {}", "✓".green(), message));
        }
        self.main_bar.inc(1);
    }

    /// Mark a task as failed.
    ///
    /// Does not increment the main progress bar, displays an error symbol.
    ///
    /// # Arguments
    ///
    /// * `task_id` - ID returned from `add_task`
    /// * `message` - Failure message to display
    pub fn fail_task(&self, task_id: usize, message: &str) {
        if let Some(pb) = self.task_bars.get(task_id) {
            pb.finish_with_message(format!("  {} {}", "✗".red(), message));
        }
    }

    /// Complete the entire progress operation.
    ///
    /// # Arguments
    ///
    /// * `message` - Final completion message
    pub fn finish(&self, message: &str) {
        self.main_bar.finish_with_message(message.to_string());
    }

    /// Check if progress bars should be shown.
    ///
    /// Returns `false` in CI environments or when output is not a TTY.
    ///
    /// # Returns
    ///
    /// `true` if progress bars should be displayed
    pub fn should_show() -> bool {
        console::user_attended() && !super::is_ci()
    }
}

impl Drop for BundleProgress {
    /// Clean up any unfinished progress bars.
    ///
    /// Ensures terminal state is properly restored even if progress is interrupted.
    fn drop(&mut self) {
        for bar in &self.task_bars {
            if !bar.is_finished() {
                bar.finish_and_clear();
            }
        }
        if !self.main_bar.is_finished() {
            self.main_bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_progress_creation() {
        // Should not panic
        let progress = BundleProgress::new(5);
        assert_eq!(progress.task_bars.len(), 0);
    }

    #[test]
    fn test_bundle_progress_add_task() {
        let mut progress = BundleProgress::new(3);
        let task1 = progress.add_task("Task 1");
        let task2 = progress.add_task("Task 2");

        assert_eq!(task1, 0);
        assert_eq!(task2, 1);
        assert_eq!(progress.task_bars.len(), 2);
    }

    #[test]
    fn test_bundle_progress_update_task() {
        let mut progress = BundleProgress::new(1);
        let task = progress.add_task("Initial");

        // Should not panic
        progress.update_task(task, "Updated");
        progress.update_task(999, "Invalid task"); // Should handle gracefully
    }

    #[test]
    fn test_bundle_progress_finish_task() {
        let mut progress = BundleProgress::new(2);
        let task = progress.add_task("Task");

        // Should not panic
        progress.finish_task(task, "Completed");
    }

    #[test]
    fn test_bundle_progress_fail_task() {
        let mut progress = BundleProgress::new(1);
        let task = progress.add_task("Task");

        // Should not panic
        progress.fail_task(task, "Failed");
    }
}
