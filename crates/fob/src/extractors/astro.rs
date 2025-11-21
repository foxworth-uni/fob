//! Astro component script extractor.
//!
//! This module implements efficient extraction of JavaScript/TypeScript from Astro
//! component frontmatter and `<script>` blocks.

use memchr::memmem;

use super::common::{ExtractedScript, Extractor, ExtractorError, ScriptContext, MAX_FILE_SIZE, MAX_SCRIPT_TAGS};

/// Astro component script extractor
#[derive(Debug, Clone, Copy)]
pub struct AstroExtractor;

impl Extractor for AstroExtractor {
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

        // First, try to extract frontmatter
        if let Some(frontmatter) = parse_frontmatter(source, &mut pointer)? {
            sources.push(frontmatter);
        }

        // Then extract all script blocks
        let mut script_count = 0;
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
        ".astro"
    }
}

/// Parses frontmatter if it exists at the start of the file.
fn parse_frontmatter<'a>(
    source_text: &'a str,
    pointer: &mut usize,
) -> Result<Option<ExtractedScript<'a>>, ExtractorError> {
    let bytes = source_text.as_bytes();

    // Skip leading whitespace
    while *pointer < bytes.len() && matches!(bytes[*pointer], b' ' | b'\t' | b'\n' | b'\r') {
        *pointer += 1;
    }

    // Check for opening ---
    if *pointer + 3 > bytes.len() || &bytes[*pointer..*pointer + 3] != b"---" {
        return Ok(None); // No frontmatter
    }

    let frontmatter_start = *pointer;
    *pointer += 3; // Skip opening ---

    // Find the newline after opening ---
    while *pointer < bytes.len() && bytes[*pointer] != b'\n' {
        *pointer += 1;
    }
    if *pointer < bytes.len() {
        *pointer += 1; // Skip the newline
    }

    let content_start = *pointer;

    // Find closing ---
    let closing_pos = match find_frontmatter_closing(bytes, *pointer) {
        Some(pos) => pos,
        None => {
            return Err(ExtractorError::UnclosedFrontmatter {
                position: frontmatter_start,
            })
        }
    };

    // Extract frontmatter content
    let source_text = &source_text[content_start..closing_pos];

    // Move pointer past closing ---
    *pointer = closing_pos + 3; // 3 = "---".len()

    // Skip to end of line after closing ---
    while *pointer < bytes.len() && bytes[*pointer] != b'\n' {
        *pointer += 1;
    }
    if *pointer < bytes.len() {
        *pointer += 1; // Skip the newline
    }

    // Frontmatter is TypeScript by default in Astro
    Ok(Some(ExtractedScript::new(
        source_text,
        content_start,
        ScriptContext::AstroFrontmatter,
        "ts",
    )))
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

    // Check for self-closing tag <script ... />
    if tag_end > 0 && bytes[tag_end - 1] == b'/' {
        // Self-closing tag, no content
        *pointer = tag_end + 1;
        return Ok(Some(ExtractedScript::new(
            "",
            tag_end + 1,
            ScriptContext::AstroScript,
            "js",
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

    // Script tags in Astro are JavaScript by default
    Ok(Some(ExtractedScript::new(
        source_text,
        content_start,
        ScriptContext::AstroScript,
        "js",
    )))
}

/// Finds the closing `---` for frontmatter.
fn find_frontmatter_closing(bytes: &[u8], start: usize) -> Option<usize> {
    let mut pos = start;

    while pos + 3 <= bytes.len() {
        // Check if we're at the start of a line
        let at_line_start = pos == 0 || bytes[pos - 1] == b'\n';

        if at_line_start && &bytes[pos..pos + 3] == b"---" {
            // Check that --- is followed by newline or end of file
            let after_dashes = pos + 3;
            if after_dashes >= bytes.len()
                || matches!(bytes[after_dashes], b'\n' | b'\r' | b' ' | b'\t')
            {
                return Some(pos);
            }
        }

        pos += 1;
    }

    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_only() {
        let astro = r#"---
const title = 'My Page'
const data = await fetch('/api').then(r => r.json())
---
<html><head><title>{title}</title></head></html>
"#;
        let extractor = AstroExtractor;
        let sources = extractor.extract(astro).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].context, ScriptContext::AstroFrontmatter);
        assert_eq!(sources[0].lang, "ts");
        assert!(sources[0].source_text.contains("const title"));
    }

    #[test]
    fn test_script_only() {
        let astro = r#"
<html>
  <body>
    <script>
      console.log('Hello, Astro!')
    </script>
  </body>
</html>
"#;
        let extractor = AstroExtractor;
        let sources = extractor.extract(astro).unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].context, ScriptContext::AstroScript);
        assert_eq!(sources[0].lang, "js");
        assert!(sources[0].source_text.contains("console.log"));
    }

    #[test]
    fn test_frontmatter_and_scripts() {
        let astro = r#"---
const pageTitle = 'Home'
---
<html>
  <head><title>{pageTitle}</title></head>
  <body>
    <script>
      console.log('Script 1')
    </script>
    <script>
      console.log('Script 2')
    </script>
  </body>
</html>
"#;
        let extractor = AstroExtractor;
        let sources = extractor.extract(astro).unwrap();
        assert_eq!(sources.len(), 3);
        assert_eq!(sources[0].context, ScriptContext::AstroFrontmatter);
        assert_eq!(sources[1].context, ScriptContext::AstroScript);
        assert_eq!(sources[2].context, ScriptContext::AstroScript);
        assert!(sources[1].source_text.contains("Script 1"));
        assert!(sources[2].source_text.contains("Script 2"));
    }

    #[test]
    fn test_no_frontmatter_or_scripts() {
        let astro = "<html><body><h1>Hello</h1></body></html>";
        let extractor = AstroExtractor;
        let sources = extractor.extract(astro).unwrap();
        assert_eq!(sources.len(), 0);
    }

    #[test]
    fn test_unclosed_frontmatter() {
        let astro = r#"---
const x = 1
<html></html>
"#;
        let extractor = AstroExtractor;
        let result = extractor.extract(astro);
        assert!(matches!(
            result,
            Err(ExtractorError::UnclosedFrontmatter { .. })
        ));
    }

    #[test]
    fn test_unclosed_script() {
        let astro = r#"<script>console.log('test')"#;
        let extractor = AstroExtractor;
        let result = extractor.extract(astro);
        assert!(matches!(result, Err(ExtractorError::UnclosedScriptTag { .. })));
    }

    #[test]
    fn test_file_too_large() {
        let large_content = "x".repeat(MAX_FILE_SIZE + 1);
        let extractor = AstroExtractor;
        let result = extractor.extract(&large_content);
        assert!(matches!(result, Err(ExtractorError::FileTooLarge { .. })));
    }
}

