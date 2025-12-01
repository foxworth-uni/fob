//! Code block node conversions

use anyhow::Result;
use markdown::mdast::Code;

use crate::codegen::{CodegenContext, JsValue, escape_js_string};

/// Metadata extracted from code fence (e.g., ```ts title="foo.ts" {1,3-5})
#[derive(Debug, Default, Clone)]
pub struct FenceMeta {
    pub title: Option<String>,
    pub line_highlights: Vec<usize>,
    pub word_highlights: Vec<String>,
}

/// Parse fence metadata string
///
/// Supported formats:
/// - `title="filename.ts"` or `title='filename.ts'`
/// - `{1,3-5,7}` for line highlights
/// - `word:foo,bar` for word highlights
pub fn parse_fence_meta(meta: &str) -> FenceMeta {
    let mut result = FenceMeta::default();
    let meta = meta.trim();

    if meta.is_empty() {
        return result;
    }

    // Parse title="..." or title='...'
    if let Some(title_start) = meta.find("title=") {
        let after_equals = &meta[title_start + 6..];
        if let Some(quote) = after_equals.chars().next() {
            if quote == '"' || quote == '\'' {
                if let Some(end_quote) = after_equals[1..].find(quote) {
                    result.title = Some(after_equals[1..end_quote + 1].to_string());
                }
            }
        }
    }

    // Parse line highlights {1,3-5,7}
    if let Some(start) = meta.find('{') {
        if let Some(end) = meta[start..].find('}') {
            let highlights_str = &meta[start + 1..start + end];
            for part in highlights_str.split(',') {
                let part = part.trim();
                if part.contains('-') {
                    // Range: 3-5
                    if let Some((start_str, end_str)) = part.split_once('-') {
                        if let (Ok(start_num), Ok(end_num)) = (
                            start_str.trim().parse::<usize>(),
                            end_str.trim().parse::<usize>(),
                        ) {
                            for line in start_num..=end_num {
                                result.line_highlights.push(line);
                            }
                        }
                    }
                } else if let Ok(line) = part.parse::<usize>() {
                    // Single line: 1
                    result.line_highlights.push(line);
                }
            }
        }
    }

    // Parse word highlights word:foo,bar
    if let Some(word_start) = meta.find("word:") {
        let after_colon = &meta[word_start + 5..];
        // Find end (space or end of string)
        let words_str = after_colon.split_whitespace().next().unwrap_or("");
        for word in words_str.split(',') {
            let word = word.trim();
            if !word.is_empty() {
                result.word_highlights.push(word.to_string());
            }
        }
    }

    result
}

/// Convert code block node to JSX
pub fn code_block_to_jsx(code: &Code, _ctx: &mut CodegenContext) -> Result<Option<JsValue>> {
    let lang = code.lang.as_deref().unwrap_or("");
    let value = &code.value;
    let meta = code.meta.as_deref().unwrap_or("");

    // Parse fence metadata if present
    let fence_meta = if !meta.is_empty() {
        parse_fence_meta(meta)
    } else {
        FenceMeta::default()
    };

    // Generate CodeBlock component call with metadata
    let mut props_parts = Vec::new();
    props_parts.push(format!("lang: \"{}\"", escape_js_string(lang)));
    props_parts.push(format!("code: \"{}\"", escape_js_string(value)));

    // Add title if present
    if let Some(title) = &fence_meta.title {
        props_parts.push(format!("title: \"{}\"", escape_js_string(title)));
    }

    // Add line highlights if present
    if !fence_meta.line_highlights.is_empty() {
        let lines: Vec<String> = fence_meta
            .line_highlights
            .iter()
            .map(|n| n.to_string())
            .collect();
        props_parts.push(format!("highlightLines: [{}]", lines.join(", ")));
    }

    // Add word highlights if present
    if !fence_meta.word_highlights.is_empty() {
        let words: Vec<String> = fence_meta
            .word_highlights
            .iter()
            .map(|w| format!("\"{}\"", escape_js_string(w)))
            .collect();
        props_parts.push(format!("highlightWords: [{}]", words.join(", ")));
    }

    props_parts.push("...props".to_string());

    // Generate JSX with conditional rendering:
    // If CodeBlock exists in components, use it; otherwise fall back to pre/code
    let jsx = format!(
        "(_components.CodeBlock ? _jsx(_components.CodeBlock, {{{}}}) : _jsx(_components.pre, {{...props, children: _jsx(_components.code, {{className: \"language-{}\", children: \"{}\"}})}}))",
        props_parts.join(", "),
        escape_js_string(lang),
        escape_js_string(value)
    );
    Ok(Some(JsValue::raw(jsx)))
}
