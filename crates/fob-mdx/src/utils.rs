//! Internal utilities for MDX compilation

/// Convert byte offset to (line, column) position in source code.
///
/// Lines and columns are 1-indexed to match standard editor conventions.
pub(crate) fn offset_to_line_col(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1;
    let mut col = 1;

    for (i, ch) in source.char_indices() {
        if i >= offset {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_line_col() {
        let source = "hello\nworld\ntest";

        // First character
        assert_eq!(offset_to_line_col(source, 0), (1, 1));

        // After "hello"
        assert_eq!(offset_to_line_col(source, 5), (1, 6));

        // Start of "world"
        assert_eq!(offset_to_line_col(source, 6), (2, 1));

        // Start of "test"
        assert_eq!(offset_to_line_col(source, 12), (3, 1));
    }
}
