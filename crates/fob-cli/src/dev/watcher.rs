//! File system watcher with debouncing for development mode.
//!
//! Watches the entire project directory and filters changes to relevant files,
//! ignoring node_modules, build artifacts, and other configured patterns.

use crate::error::{CliError, Result};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

/// File change event type.
#[derive(Debug, Clone)]
pub enum FileChange {
    /// File was modified
    Modified(PathBuf),
    /// File was created
    Created(PathBuf),
    /// File was removed
    Removed(PathBuf),
}

impl FileChange {
    /// Get the path affected by this change.
    pub fn path(&self) -> &Path {
        match self {
            FileChange::Modified(p) | FileChange::Created(p) | FileChange::Removed(p) => p,
        }
    }
}

/// File watcher with debouncing and filtering.
///
/// Watches a directory recursively and sends change events through a channel.
/// Debouncing prevents rapid successive events from causing multiple rebuilds.
pub struct FileWatcher {
    /// Underlying notify watcher
    _watcher: RecommendedWatcher,
    /// Root directory being watched
    root: PathBuf,
    /// Patterns to ignore (e.g., "node_modules", "*.log")
    #[allow(dead_code)]
    ignore_patterns: Vec<String>,
}

impl FileWatcher {
    /// Create a new file watcher.
    ///
    /// # Arguments
    ///
    /// * `root` - Root directory to watch recursively
    /// * `ignore_patterns` - Patterns to ignore (glob-style)
    /// * `debounce_ms` - Debounce delay in milliseconds
    ///
    /// # Returns
    ///
    /// Tuple of (FileWatcher, receiver for change events)
    ///
    /// # Errors
    ///
    /// Returns error if watcher cannot be created or directory doesn't exist
    pub fn new(
        root: PathBuf,
        ignore_patterns: Vec<String>,
        debounce_ms: u64,
    ) -> Result<(Self, mpsc::Receiver<FileChange>)> {
        // Validate root directory exists
        if !root.exists() {
            return Err(CliError::FileNotFound(root));
        }

        let (tx, rx) = mpsc::channel(100);

        // Create debouncer to batch rapid changes
        let debounce_duration = Duration::from_millis(debounce_ms);
        let mut last_event: Option<(PathBuf, Instant)> = None;
        let ignore_patterns_clone = ignore_patterns.clone();
        let root_clone = root.clone();

        // Create watcher with event handler
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                // Process each path in the event
                for path in &event.paths {
                    // Skip if path should be ignored
                    if Self::should_ignore(path, &root_clone, &ignore_patterns_clone) {
                        continue;
                    }

                    // Debounce: skip if same file changed within debounce window
                    let now = Instant::now();
                    if let Some((last_path, last_time)) = &last_event {
                        if last_path == path && now.duration_since(*last_time) < debounce_duration {
                            continue;
                        }
                    }

                    last_event = Some((path.clone(), now));

                    // Convert notify event to our FileChange type
                    let change = match event.kind {
                        notify::EventKind::Create(_) => FileChange::Created(path.clone()),
                        notify::EventKind::Modify(_) => FileChange::Modified(path.clone()),
                        notify::EventKind::Remove(_) => FileChange::Removed(path.clone()),
                        _ => continue,
                    };

                    // Send event (non-blocking)
                    let _ = tx.blocking_send(change);
                }
            }
        })
        .map_err(|e| CliError::Watch(e))?;

        // Start watching the root directory
        watcher
            .watch(&root, RecursiveMode::Recursive)
            .map_err(|e| CliError::Watch(e))?;

        Ok((
            Self {
                _watcher: watcher,
                root,
                ignore_patterns,
            },
            rx,
        ))
    }

    /// Check if a path should be ignored.
    ///
    /// # Security
    ///
    /// - Prevents watching system directories
    /// - Validates paths are within project root
    fn should_ignore(path: &Path, root: &Path, ignore_patterns: &[String]) -> bool {
        // Security: Only watch files within root
        if !path.starts_with(root) {
            return true;
        }

        // Get relative path for pattern matching
        let rel_path = match path.strip_prefix(root) {
            Ok(p) => p,
            Err(_) => return true,
        };

        let path_str = rel_path.to_string_lossy();

        // Check each ignore pattern
        for pattern in ignore_patterns {
            // Simple pattern matching (could be enhanced with glob crate)
            if pattern.starts_with('*') {
                // Extension pattern like "*.log"
                let ext = pattern.trim_start_matches('*');
                if path_str.ends_with(ext) {
                    return true;
                }
            } else if path_str.starts_with(pattern) || path_str.contains(&format!("/{}", pattern)) {
                // Directory pattern like "node_modules"
                return true;
            }
        }

        // Ignore hidden files and directories
        for component in rel_path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if name.starts_with('.') && name != "." && name != ".." {
                    return true;
                }
            }
        }

        false
    }

    /// Get the root directory being watched.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_ignore_node_modules() {
        let root = PathBuf::from("/project");
        let patterns = vec!["node_modules".to_string()];

        let path = PathBuf::from("/project/node_modules/package/index.js");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));

        let path = PathBuf::from("/project/src/index.js");
        assert!(!FileWatcher::should_ignore(&path, &root, &patterns));
    }

    #[test]
    fn test_should_ignore_extension() {
        let root = PathBuf::from("/project");
        let patterns = vec!["*.log".to_string()];

        let path = PathBuf::from("/project/debug.log");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));

        let path = PathBuf::from("/project/src/index.js");
        assert!(!FileWatcher::should_ignore(&path, &root, &patterns));
    }

    #[test]
    fn test_should_ignore_hidden_files() {
        let root = PathBuf::from("/project");
        let patterns = vec![];

        let path = PathBuf::from("/project/.git/config");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));

        let path = PathBuf::from("/project/.env");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));

        let path = PathBuf::from("/project/src/.hidden/file.js");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));
    }

    #[test]
    fn test_should_ignore_outside_root() {
        let root = PathBuf::from("/project");
        let patterns = vec![];

        let path = PathBuf::from("/other/file.js");
        assert!(FileWatcher::should_ignore(&path, &root, &patterns));
    }

    #[test]
    fn test_file_change_path() {
        let path = PathBuf::from("/project/src/index.js");

        let change = FileChange::Modified(path.clone());
        assert_eq!(change.path(), path.as_path());

        let change = FileChange::Created(path.clone());
        assert_eq!(change.path(), path.as_path());

        let change = FileChange::Removed(path.clone());
        assert_eq!(change.path(), path.as_path());
    }
}
