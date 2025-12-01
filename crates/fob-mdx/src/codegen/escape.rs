//! String escaping and JavaScript identifier utilities

/// Escape a string for safe inclusion in JavaScript string literals
///
/// Handles all special characters including Unicode line/paragraph separators
/// which can break JavaScript parsers even inside string literals.
pub fn escape_js_string(text: &str) -> String {
    // Pre-allocate: worst case is every char needs escaping (2x size)
    let mut result = String::with_capacity(text.len() * 2);

    for ch in text.chars() {
        match ch {
            // Standard escapes (JSON-compatible)
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),

            // Additional JavaScript escapes
            '\x08' => result.push_str("\\b"), // backspace
            '\x0C' => result.push_str("\\f"), // form feed

            // CRITICAL: Unicode line terminators break JS even in strings
            '\u{2028}' => result.push_str("\\u2028"), // Line Separator
            '\u{2029}' => result.push_str("\\u2029"), // Paragraph Separator

            // Control characters (C0 range: 0x00-0x1F except handled above)
            ch if ch.is_control() && !matches!(ch, '\n' | '\r' | '\t' | '\x08' | '\x0C') => {
                // Use Unicode escape for safety
                result.push_str(&format!("\\u{:04x}", ch as u32));
            }

            // Normal characters
            _ => result.push(ch),
        }
    }

    result
}

/// Check if a string is a valid JavaScript identifier
///
/// Valid identifiers:
/// - Start with letter, $, or _
/// - Continue with letters, digits, $, or _
/// - Not a reserved word
///
/// This allows us to use unquoted object properties when safe.
pub fn is_valid_identifier(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Reserved words that can't be used as unquoted properties
    const RESERVED: &[&str] = &[
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "export",
        "extends",
        "finally",
        "for",
        "function",
        "if",
        "import",
        "in",
        "instanceof",
        "new",
        "return",
        "super",
        "switch",
        "this",
        "throw",
        "try",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
        "let",
        "static",
        "enum",
        "await",
        "implements",
        "interface",
        "package",
        "private",
        "protected",
        "public",
    ];

    if RESERVED.contains(&name) {
        return false;
    }

    let mut chars = name.chars();

    // First character: must be letter, $, or _
    match chars.next() {
        Some(c) if c.is_alphabetic() || c == '$' || c == '_' => {}
        _ => return false,
    }

    // Remaining characters: letter, digit, $, or _
    chars.all(|c| c.is_alphanumeric() || c == '$' || c == '_')
}

#[cfg(test)]
mod escape_tests {
    use super::*;

    #[test]
    fn test_basic_escapes() {
        assert_eq!(escape_js_string("hello"), "hello");
        assert_eq!(escape_js_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_js_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_quotes_and_backslash() {
        assert_eq!(escape_js_string("say \"hi\""), "say \\\"hi\\\"");
        assert_eq!(escape_js_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_unicode_separators() {
        // U+2028 and U+2029 MUST be escaped or they break JS parsing
        assert_eq!(
            escape_js_string("line\u{2028}separator"),
            "line\\u2028separator"
        );
        assert_eq!(
            escape_js_string("para\u{2029}separator"),
            "para\\u2029separator"
        );
    }

    #[test]
    fn test_control_characters() {
        assert_eq!(escape_js_string("null\x00char"), "null\\u0000char");
        assert_eq!(escape_js_string("bell\x07char"), "bell\\u0007char");
        assert_eq!(escape_js_string("back\x08space"), "back\\bspace"); // Backspace gets \b
    }

    #[test]
    fn test_xss_attempts() {
        // These should be safely escaped, not allow injection
        assert_eq!(escape_js_string("</script>"), "</script>"); // Stays as-is (in string context)
        assert_eq!(escape_js_string("${injection}"), "${injection}"); // Template literal syntax safe in normal strings
        assert_eq!(escape_js_string("`backtick`"), "`backtick`"); // Backticks are safe in double-quoted strings
    }

    #[test]
    fn test_empty_and_large() {
        assert_eq!(escape_js_string(""), "");
        let large = "a".repeat(10000);
        assert_eq!(escape_js_string(&large).len(), 10000);
    }
}

#[cfg(test)]
mod identifier_tests {
    use super::*;

    #[test]
    fn test_valid_identifiers() {
        assert!(is_valid_identifier("foo"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("$jquery"));
        assert!(is_valid_identifier("camelCase"));
        assert!(is_valid_identifier("CONSTANT"));
        assert!(is_valid_identifier("value123"));
    }

    #[test]
    fn test_invalid_identifiers() {
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier("123start"));
        assert!(!is_valid_identifier("kebab-case"));
        assert!(!is_valid_identifier("has space"));
        assert!(!is_valid_identifier("has.dot"));
        assert!(!is_valid_identifier("return")); // Reserved word
        assert!(!is_valid_identifier("class")); // Reserved word
    }
}
