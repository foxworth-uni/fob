use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Byte range within a source file used for diagnostics and tooling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub file: PathBuf,
    pub start: u32,
    pub end: u32,
}

impl SourceSpan {
    /// Construct a new span from a path and byte offsets.
    pub fn new(file: impl AsRef<Path>, start: u32, end: u32) -> Self {
        debug_assert!(start <= end, "span start must not exceed end");

        Self {
            file: file.as_ref().to_path_buf(),
            start,
            end,
        }
    }

    /// Length of the span in bytes.
    pub fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Returns true when the span has zero width.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Merge two spans that reference the same file.
    pub fn merge(&self, other: &Self) -> Option<Self> {
        if self.file != other.file {
            return None;
        }

        Some(Self {
            file: self.file.clone(),
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        })
    }

    /// Convert the byte offset to a 1-indexed `(line, column)` pair.
    pub fn to_line_col(&self, source: &str) -> (usize, usize) {
        let mut line = 1usize;
        let mut col = 1usize;

        for (idx, ch) in source.chars().enumerate() {
            if idx >= self.start as usize {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Check whether the span contains a byte offset.
    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}-{}", self.file.display(), self.start, self.end)
    }
}
