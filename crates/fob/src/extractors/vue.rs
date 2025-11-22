//! Vue Single File Component (SFC) script extractor.
//!
//! This module implements efficient extraction of JavaScript/TypeScript from Vue SFC
//! `<script>` blocks.

use memchr::memmem;

use super::common::{
    ExtractedScript, Extractor, ExtractorError, ScriptContext, MAX_FILE_SIZE, MAX_SCRIPT_TAGS,
};

/// Vue SFC script extractor
#[derive(Debug, Clone, Copy)]
pub struct VueExtractor;

impl Extractor for VueExtractor {
    fn extract<'a>(&self, source: &'a str) -> Result<Vec<ExtractedScript<'a>>, ExtractorError> {
        // Enforce file size limit
        if source.len() > MAX_FILE_SIZE {
            return Err(ExtractorError::FileTooLarge {
                size: source.len(),
                max: MAX_FILE_SIZE,
            });
        }

        let mut sources = Vec::new();
        let mut pointer = 0;
        let mut script_count = 0;

        // Extract all script blocks
        while let Some(script) = parse_script(source, &mut pointer)? {
            sources.push(script);
            script_count += 1;

            // Enforce script tag count limit
            if script_count > MAX_SCRIPT_TAGS {
                return Err(ExtractorError::TooManyScriptTags {
                    count: script_count,
                    max: MAX_SCRIPT_TAGS,
                });
            }
        }

        Ok(sources)
    }

    fn file_extension(&self) -> &'static str {
        ".vue"
    }
}

/// Parses a single script block starting from the given position.
fn parse_script<'a>(
    source_text: &'a str,
    pointer: &mut usize,
) -> Result<Option<ExtractedScript<'a>>, ExtractorError> {
    let bytes = source_text.as_bytes();

    // Find the start of a <script tag
    let script_start = match find_script_start(bytes, *pointer) {
        Some(pos) => pos,
        None => return Ok(None), // No more script tags
    };

    // Move pointer past "<script"
    *pointer = script_start + 7; // 7 = "<script".len()

    // Check if this is a script tag (not "scripts" or "scripting")
    if *pointer < bytes.len() {
        let next_char = bytes[*pointer];
        if !matches!(next_char, b' ' | b'\t' | b'\n' | b'\r' | b'>' | b'/') {
            // Not a script tag, keep searching
            return parse_script(source_text, pointer);
        }
    }

    // Find the end of the opening tag (the closing >)
    let tag_end = match find_script_closing_angle(bytes, *pointer) {
        Some(pos) => pos,
        None => {
            return Err(ExtractorError::UnclosedScriptTag {
                position: script_start,
            })
        }
    };

    // Extract the tag attributes (between "<script" and ">")
    let tag_content = &source_text[*pointer..tag_end];

    // Parse attributes
    let is_setup = tag_content.contains("setup");
    let lang = extract_lang_attribute(tag_content);

    // Check for self-closing tag <script ... />
    if tag_end > 0 && bytes[tag_end - 1] == b'/' {
        // Self-closing tag, no content
        *pointer = tag_end + 1;
        return Ok(Some(ExtractedScript::new(
            "",
            tag_end + 1,
            if is_setup {
                ScriptContext::VueSetup
            } else {
                ScriptContext::VueRegular
            },
            lang,
        )));
    }

    // Move pointer past the closing >
    *pointer = tag_end + 1;
    let content_start = *pointer;

    // Find the closing </script> tag
    let script_end = match find_script_end(bytes, *pointer) {
        Some(pos) => pos,
        None => {
            return Err(ExtractorError::UnclosedScriptTag {
                position: script_start,
            })
        }
    };

    // Extract the script content
    let source_text = &source_text[content_start..script_end];

    // Move pointer past the closing </script>
    *pointer = script_end + 9; // 9 = "</script>".len()

    Ok(Some(ExtractedScript::new(
        source_text,
        content_start,
        if is_setup {
            ScriptContext::VueSetup
        } else {
            ScriptContext::VueRegular
        },
        lang,
    )))
}

/// Finds the start of a `<script` tag using memchr.
fn find_script_start(bytes: &[u8], start: usize) -> Option<usize> {
    let search_slice = &bytes[start..];
    memmem::find(search_slice, b"<script").map(|pos| start + pos)
}

/// Finds the closing `>` of a script tag, handling quoted attributes.
fn find_script_closing_angle(bytes: &[u8], start: usize) -> Option<usize> {
    let mut in_quote = false;
    let mut quote_char = 0u8;

    for (i, &byte) in bytes[start..].iter().enumerate() {
        match byte {
            b'"' | b'\'' => {
                if !in_quote {
                    in_quote = true;
                    quote_char = byte;
                } else if byte == quote_char {
                    in_quote = false;
                }
            }
            b'>' if !in_quote => return Some(start + i),
            _ => {}
        }
    }

    None
}

/// Finds the closing `</script>` tag.
fn find_script_end(bytes: &[u8], start: usize) -> Option<usize> {
    let search_slice = &bytes[start..];
    memmem::find(search_slice, b"</script>").map(|pos| start + pos)
}

/// Extracts the `lang` attribute value from a script tag.
fn extract_lang_attribute(tag_content: &str) -> &str {
    // Find "lang=" or 'lang='
    if let Some(lang_pos) = tag_content.find("lang=") {
        let after_equals = &tag_content[lang_pos + 5..];

        // Skip whitespace
        let after_equals = after_equals.trim_start();

        if after_equals.is_empty() {
            return "js";
        }

        // Check for quoted value
        let quote_char = after_equals.chars().next().unwrap();
        if quote_char == '"' || quote_char == '\'' {
            // Find closing quote
            if let Some(end_quote) = after_equals[1..].find(quote_char) {
                return &after_equals[1..=end_quote];
            }
        } else {
            // Unquoted value (non-standard but handle it)
            let end = after_equals
                .find(|c: char| c.is_whitespace() || c == '>')
                .unwrap_or(after_equals.len());
            return &after_equals[..end];
        }
    }

    "js" // Default to JavaScript
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_script() {
        let vue = r#"
<template><div>Hello</div></template>
<script>
export default { name: 'Test' }
</script>
"#;
        let extractor = VueExtractor;
        let sources = extractor.extract(vue).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].context, ScriptContext::VueRegular);
        assert_eq!(sources[0].lang, "js");
        assert!(sources[0].source_text.contains("export default"));
    }

    #[test]
    fn test_script_setup() {
        let vue = r#"
<script setup>
import { ref } from 'vue'
const count = ref(0)
</script>
"#;
        let extractor = VueExtractor;
        let sources = extractor.extract(vue).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].context, ScriptContext::VueSetup);
        assert!(sources[0].source_text.contains("const count"));
    }

    #[test]
    fn test_typescript() {
        let vue = r#"
<script lang="ts">
export default defineComponent({ name: 'Test' })
</script>
"#;
        let extractor = VueExtractor;
        let sources = extractor.extract(vue).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].lang, "ts");
    }

    #[test]
    fn test_multiple_scripts() {
        let vue = r#"
<script>
export default { name: 'Test' }
</script>
<script setup lang="ts">
const count = ref<number>(0)
</script>
"#;
        let extractor = VueExtractor;
        let sources = extractor.extract(vue).unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0].context, ScriptContext::VueRegular);
        assert_eq!(sources[1].context, ScriptContext::VueSetup);
        assert_eq!(sources[1].lang, "ts");
    }

    #[test]
    fn test_no_script() {
        let vue = "<template><div>Hello</div></template>";
        let extractor = VueExtractor;
        let sources = extractor.extract(vue).unwrap();
        assert_eq!(sources.len(), 0);
    }

    #[test]
    fn test_file_too_large() {
        let large_content = "x".repeat(MAX_FILE_SIZE + 1);
        let extractor = VueExtractor;
        let result = extractor.extract(&large_content);
        assert!(matches!(result, Err(ExtractorError::FileTooLarge { .. })));
    }
}
