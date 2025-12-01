//! JavaScript value types with proper escaping

use super::escape::escape_js_string;

/// Represents a JavaScript value with proper escaping semantics
///
/// This type prevents double-escaping bugs by tracking whether content
/// is already valid JavaScript code vs raw text that needs escaping.
#[derive(Debug, Clone, PartialEq)]
pub enum JsValue {
    /// Raw JavaScript code (already valid, no escaping needed)
    /// Examples: `_jsx(...)`, `props.name`, `[1, 2, 3]`
    Raw(String),

    /// Plain text that needs escaping when converted to JS
    /// Examples: user content, markdown text
    Text(String),

    /// Array of mixed values (for children arrays)
    Array(Vec<JsValue>),
}

impl JsValue {
    /// Create a raw JavaScript value (assumes content is valid JS)
    pub fn raw(s: impl Into<String>) -> Self {
        Self::Raw(s.into())
    }

    /// Create a text value (will be escaped when serialized)
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create an array value
    pub fn array(values: Vec<JsValue>) -> Self {
        Self::Array(values)
    }

    /// Convert to JavaScript code
    pub fn to_js(&self) -> String {
        match self {
            JsValue::Raw(s) => s.clone(),
            JsValue::Text(s) => format!("\"{}\"", escape_js_string(s)),
            JsValue::Array(items) => {
                let elements: Vec<String> = items.iter().map(|v| v.to_js()).collect();
                format!("[{}]", elements.join(", "))
            }
        }
    }
}

#[cfg(test)]
mod jsvalue_tests {
    use super::*;

    #[test]
    fn test_raw_values() {
        assert_eq!(JsValue::raw("_jsx(Foo, {})").to_js(), "_jsx(Foo, {})");
        assert_eq!(JsValue::raw("props.name").to_js(), "props.name");
    }

    #[test]
    fn test_text_values() {
        assert_eq!(JsValue::text("hello").to_js(), "\"hello\"");
        assert_eq!(JsValue::text("say \"hi\"").to_js(), "\"say \\\"hi\\\"\"");
        assert_eq!(JsValue::text("line\nbreak").to_js(), "\"line\\nbreak\"");
    }

    #[test]
    fn test_array_values() {
        let arr = JsValue::array(vec![
            JsValue::text("hello"),
            JsValue::raw("props.name"),
            JsValue::text("world"),
        ]);
        assert_eq!(arr.to_js(), "[\"hello\", props.name, \"world\"]");
    }

    #[test]
    fn test_no_double_escaping() {
        // Text value already containing quotes should not double-escape
        let text = "already\\nescaped";
        let value = JsValue::text(text);
        assert_eq!(value.to_js(), "\"already\\\\nescaped\"");
    }
}
