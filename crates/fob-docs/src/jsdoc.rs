use crate::model::{JsDocTag, ParameterDoc};

/// Structured representation of a parsed JSDoc comment.
#[derive(Debug, Default, Clone)]
pub struct ParsedJsDoc {
    /// Summary text before any tags.
    pub summary: Option<String>,
    /// Parameter information derived from `@param` tags.
    pub parameters: Vec<ParameterDoc>,
    /// Return value description from `@return` / `@returns`.
    pub returns: Option<String>,
    /// Deprecated message from `@deprecated`.
    pub deprecated: Option<String>,
    /// Example snippets collected from `@example`.
    pub examples: Vec<String>,
    /// Arbitrary tags that were not converted into typed fields.
    pub tags: Vec<JsDocTag>,
    /// Whether the comment contained `@internal`.
    pub is_internal: bool,
}

impl ParsedJsDoc {
    /// Returns `true` if the comment does not contain any meaningful data.
    pub fn is_empty(&self) -> bool {
        self.summary
            .as_ref()
            .map(|s| s.trim().is_empty())
            .unwrap_or(true)
            && self.parameters.is_empty()
            && self.returns.is_none()
            && self.deprecated.is_none()
            && self.examples.is_empty()
            && self.tags.is_empty()
    }
}

/// Parse a JSDoc block (without comment delimiters) into structured data.
///
/// The parser is intentionally lightweight and aims to handle the most common tag
/// patterns without requiring a full-fledged JSDoc grammar.
pub fn parse_jsdoc(raw: &str) -> ParsedJsDoc {
    let mut summary_lines = Vec::new();
    let mut parameters = Vec::new();
    let mut returns = None;
    let mut deprecated = None;
    let mut examples = Vec::new();
    let mut tags = Vec::new();
    let mut is_internal = false;

    let lines = normalize_lines(raw);
    let mut idx = 0;
    while idx < lines.len() {
        let line = &lines[idx];

        if let Some(rest) = line.strip_prefix('@') {
            let (tag, payload) = split_tag_payload(rest);
            match tag {
                "param" => {
                    if let Some(param) = parse_param(payload) {
                        parameters.push(param);
                    }
                }
                "returns" | "return" => {
                    let (type_hint, description) = parse_type_and_rest(payload);
                    let mut value = String::new();
                    if let Some(ty) = type_hint {
                        value.push_str(&format!("{} ", ty));
                    }
                    if let Some(desc) = description {
                        value.push_str(&desc);
                    }
                    let trimmed = value.trim().to_string();
                    if !trimmed.is_empty() {
                        returns = Some(trimmed);
                    }
                }
                "deprecated" => {
                    let value = payload.trim();
                    if !value.is_empty() {
                        deprecated = Some(value.to_string());
                    } else {
                        deprecated = Some("Deprecated".to_string());
                    }
                }
                "example" => {
                    let mut example_lines = Vec::new();
                    if !payload.trim().is_empty() {
                        example_lines.push(payload.trim().to_string());
                    }
                    idx += 1;
                    while idx < lines.len() {
                        let peek = &lines[idx];
                        if peek.starts_with('@') {
                            idx -= 1; // compensate for upcoming increment
                            break;
                        }
                        if !peek.is_empty() || !example_lines.is_empty() {
                            example_lines.push(peek.to_string());
                        }
                        idx += 1;
                    }
                    let example = example_lines.join("\n").trim().to_string();
                    if !example.is_empty() {
                        examples.push(example);
                    }
                }
                "internal" => {
                    is_internal = true;
                }
                other => {
                    let mut tag = JsDocTag::new(other.to_string());
                    if let Some(payload) = payload.trim().strip_prefix('{') {
                        // rough heuristic: treat `{type}` prefix specially
                        if let Some((ty, rest)) = payload.split_once('}') {
                            if !ty.trim().is_empty() {
                                tag.type_hint = Some(ty.trim().to_string());
                            }
                            let rest = rest.trim();
                            if !rest.is_empty() {
                                tag.description = Some(rest.to_string());
                            }
                        } else {
                            tag.description = Some(payload.trim().to_string());
                        }
                    } else if !payload.trim().is_empty() {
                        tag.description = Some(payload.trim().to_string());
                    }
                    tags.push(tag);
                }
            }
        } else {
            summary_lines.push(line.to_string());
        }

        idx += 1;
    }

    ParsedJsDoc {
        summary: compose_summary(summary_lines),
        parameters,
        returns,
        deprecated,
        examples,
        tags,
        is_internal,
    }
}

fn normalize_lines(raw: &str) -> Vec<String> {
    raw.lines()
        .map(|line| {
            let line = line.trim();
            let line = line.strip_prefix('*').unwrap_or(line);
            line.trim().to_string()
        })
        .collect()
}

fn compose_summary(lines: Vec<String>) -> Option<String> {
    let summary = lines
        .into_iter()
        .skip_while(|line| line.trim().is_empty())
        .collect::<Vec<_>>();
    if summary.is_empty() {
        None
    } else {
        Some(summary.join(" ").trim().to_string())
    }
}

fn split_tag_payload(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, char::is_whitespace);
    let tag = parts.next().unwrap_or("");
    let payload = parts.next().unwrap_or("").trim();
    (tag, payload)
}

fn parse_param(payload: &str) -> Option<ParameterDoc> {
    let (type_hint, rest) = parse_type_and_rest(payload);
    let rest = rest.unwrap_or_default();
    let mut parts = rest.splitn(2, char::is_whitespace);
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }

    let mut param = ParameterDoc::new(name.trim_matches(|c| c == '[' || c == ']'));
    if let Some(ty) = type_hint {
        param.type_hint = Some(ty);
    }
    if let Some(description) = parts.next() {
        let desc = description.trim();
        if !desc.is_empty() {
            param.description = Some(desc.to_string());
        }
    }
    Some(param)
}

fn parse_type_and_rest(payload: &str) -> (Option<String>, Option<String>) {
    let trimmed = payload.trim();
    if let Some(stripped) = trimmed.strip_prefix('{') {
        if let Some((ty, rest)) = stripped.split_once('}') {
            let ty = ty.trim();
            let rest = rest.trim();
            let ty = (!ty.is_empty()).then(|| ty.to_string());
            let rest = (!rest.is_empty()).then(|| rest.to_string());
            return (ty, rest);
        }
    }
    (None, Some(trimmed.to_string()))
}
