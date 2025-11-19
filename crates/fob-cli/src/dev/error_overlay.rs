//! Error overlay generator for development mode.
//!
//! Creates an HTML error page displayed in the browser when builds fail.
//! Auto-dismisses and reloads when the next build succeeds.

/// Generate an HTML error overlay page.
///
/// Creates a styled error page that displays build errors in the browser.
/// The overlay includes:
/// - Dark theme for reduced eye strain
/// - Properly escaped error messages
/// - Retry button to manually trigger rebuild
///
/// # Arguments
///
/// * `error` - Error message to display
///
/// # Returns
///
/// HTML string ready to serve, or an error if generation fails
///
/// # Security
///
/// - HTML-escapes error messages to prevent XSS
/// - No inline JavaScript execution from error content
/// - CSP-compatible (no eval, unsafe-inline limited to style)
pub fn generate_error_overlay(error: &str) -> Result<String, String> {
    use fob_gen::{Allocator, HtmlBuilder};
    
    let allocator = Allocator::default();
    let html_builder = HtmlBuilder::new(&allocator);
    
    html_builder.error_overlay(error)
        .map_err(|e| format!("Failed to generate error overlay: {}", e))
}

/// HTML-escape a string to prevent XSS attacks.
///
/// Escapes the following characters:
/// - `&` -> `&amp;`
/// - `<` -> `&lt;`
/// - `>` -> `&gt;`
/// - `"` -> `&quot;`
/// - `'` -> `&#x27;`
///
#[cfg(test)]
mod tests {
    use super::*;

    /// Escape HTML special characters to prevent XSS attacks.
    ///
    /// Converts the following characters:
    /// - `&` -> `&amp;`
    /// - `<` -> `&lt;`
    /// - `>` -> `&gt;`
    /// - `"` -> `&quot;`
    /// - `'` -> `&#x27;`
    ///
    /// # Security
    ///
    /// This is critical for preventing XSS when displaying error messages
    /// that might contain user input or file paths with special characters.
    fn html_escape(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '&' => "&amp;".to_string(),
                '<' => "&lt;".to_string(),
                '>' => "&gt;".to_string(),
                '"' => "&quot;".to_string(),
                '\'' => "&#x27;".to_string(),
                _ => c.to_string(),
            })
            .collect()
    }

    #[test]
    fn test_html_escape_ampersand() {
        assert_eq!(html_escape("a & b"), "a &amp; b");
    }

    #[test]
    fn test_html_escape_angle_brackets() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
    }

    #[test]
    fn test_html_escape_quotes() {
        assert_eq!(
            html_escape(r#"He said "hello""#),
            "He said &quot;hello&quot;"
        );
        assert_eq!(html_escape("It's working"), "It&#x27;s working");
    }

    #[test]
    fn test_html_escape_combined() {
        let input = r#"Error in <Component attr="value" & 'test'>"#;
        let expected =
            r#"Error in &lt;Component attr=&quot;value&quot; &amp; &#x27;test&#x27;&gt;"#;
        assert_eq!(html_escape(input), expected);
    }

    #[test]
    fn test_html_escape_no_special_chars() {
        let input = "Normal error message";
        assert_eq!(html_escape(input), input);
    }

    #[test]
    fn test_generate_error_overlay_contains_escaped_error() {
        let error = "<script>alert('xss')</script>";
        let html = generate_error_overlay(error).expect("HTML generation should succeed");

        // Should contain escaped version
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn test_generate_error_overlay_structure() {
        let html = generate_error_overlay("Test error").expect("HTML generation should succeed");

        // Check required elements
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Build Error"));
        assert!(html.contains("Test error"));
        assert!(html.contains("Retry Build"));
        assert!(html.contains("/__fob_sse__"));
    }
}
