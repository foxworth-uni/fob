//! Prompt templates for LLM-enhanced documentation generation.

use crate::{ExportedSymbol, ParameterDoc, SymbolKind};

/// Context provided to the LLM for generating documentation.
#[derive(Debug, Clone)]
pub struct EnhancementContext {
    /// File path for reference.
    pub file_path: String,

    /// Optional surrounding code snippet.
    pub surrounding_code: Option<String>,
}

/// Prompt builder for generating LLM documentation requests.
pub struct PromptBuilder;

impl PromptBuilder {
    /// Builds a prompt for the given symbol and context.
    pub fn build_prompt(symbol: &ExportedSymbol, context: &EnhancementContext) -> String {
        match symbol.kind {
            SymbolKind::Function => Self::function_prompt(symbol, context),
            SymbolKind::Class => Self::class_prompt(symbol, context),
            SymbolKind::Interface => Self::interface_prompt(symbol, context),
            SymbolKind::TypeAlias => Self::type_prompt(symbol, context),
            SymbolKind::Enum => Self::enum_prompt(symbol, context),
            SymbolKind::Variable => Self::variable_prompt(symbol, context),
            _ => Self::generic_prompt(symbol, context),
        }
    }

    fn function_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let params = Self::format_parameters(&symbol.parameters);
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate concise, accurate documentation for this TypeScript/JavaScript function.

Function: {name}
File: {file}
Parameters: {params}
Return Type: {returns}
Existing JSDoc: {existing}

IMPORTANT: Return ONLY a JSON object with no additional text before or after.

Generate a JSON response with this exact structure:
{{
  "explanation": "2-3 sentence clear explanation of what this function does, why it's useful, and when to use it",
  "examples": [
    "// Example 1\nconst result = {name}({example_call});",
    "// Example 2 (if applicable)\nconst result2 = {name}({example_call2});"
  ],
  "bestPractices": [
    "Best practice or gotcha 1",
    "Best practice or gotcha 2"
  ]
}}

Do NOT include any preamble like "Here is the JSON" or wrap it in markdown code blocks.

Requirements:
- explanation: Focus on WHAT it does and WHY/WHEN to use it (not HOW - that's obvious from code)
- examples: Must be complete, runnable TypeScript code with realistic values
- bestPractices: 1-3 actionable tips about using this function correctly
- Output ONLY valid JSON (no markdown, no code blocks, no extra text)
- Keep it concise but informative"#,
            name = symbol.name,
            file = ctx.file_path,
            params = params,
            returns = symbol.returns.as_deref().unwrap_or("unknown"),
            existing = existing_doc,
            example_call = Self::generate_example_call(&symbol.parameters),
            example_call2 = Self::generate_example_call(&symbol.parameters),
        )
    }

    fn class_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate concise documentation for this TypeScript/JavaScript class.

Class: {name}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "2-3 sentence explanation of what this class represents, its purpose, and typical use cases",
  "examples": [
    "// Creating and using {name}\nconst instance = new {name}();\ninstance.someMethod();"
  ],
  "bestPractices": [
    "When to use this class",
    "Important usage considerations"
  ]
}}

Requirements:
- explanation: Describe the class's purpose and role in the system
- examples: Show instantiation and basic usage
- bestPractices: Key points about using this class effectively
- Output ONLY valid JSON"#,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    fn interface_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate documentation for this TypeScript interface.

Interface: {name}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "2-3 sentence explanation of what this interface represents and when it's used",
  "examples": [
    "// Implementing {name}\nconst example: {name} = {{\n  // properties\n}};"
  ],
  "bestPractices": [
    "When to use this interface",
    "Common patterns"
  ]
}}

Requirements:
- explanation: Describe what this interface models
- examples: Show a realistic implementation
- bestPractices: Usage patterns and considerations
- Output ONLY valid JSON"#,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    fn type_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate documentation for this TypeScript type alias.

Type: {name}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "2 sentence explanation of what this type represents and when to use it",
  "examples": [
    "// Using {name}\nconst value: {name} = ...;"
  ],
  "bestPractices": [
    "Type usage tip"
  ]
}}

Requirements:
- explanation: Concise description of the type
- examples: Show practical usage
- bestPractices: 1-2 tips
- Output ONLY valid JSON"#,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    fn enum_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate documentation for this TypeScript enum.

Enum: {name}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "2 sentence explanation of what this enum represents",
  "examples": [
    "// Using {name}\nconst value = {name}.SOME_VALUE;"
  ],
  "bestPractices": [
    "Enum usage tip"
  ]
}}

Output ONLY valid JSON."#,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    fn variable_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate documentation for this exported variable/constant.

Variable: {name}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "1-2 sentence explanation of what this variable/constant is and its purpose",
  "examples": [
    "// Using {name}\nimport {{ {name} }} from './{file}';\nconsole.log({name});"
  ],
  "bestPractices": [
    "Usage tip"
  ]
}}

Output ONLY valid JSON."#,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    fn generic_prompt(symbol: &ExportedSymbol, ctx: &EnhancementContext) -> String {
        let existing_doc = symbol
            .summary
            .as_deref()
            .unwrap_or("No existing documentation");

        format!(
            r#"You are a technical documentation expert. Generate documentation for this exported symbol.

Symbol: {name}
Kind: {:?}
File: {file}
Existing JSDoc: {existing}

Generate a JSON response:
{{
  "explanation": "2 sentence explanation of this symbol",
  "examples": [
    "// Usage example"
  ],
  "bestPractices": [
    "Usage tip"
  ]
}}

Output ONLY valid JSON."#,
            symbol.kind,
            name = symbol.name,
            file = ctx.file_path,
            existing = existing_doc,
        )
    }

    /// Formats parameters for display in prompts.
    fn format_parameters(params: &[ParameterDoc]) -> String {
        if params.is_empty() {
            return "none".to_string();
        }

        params
            .iter()
            .map(|p| {
                let type_hint = p.type_hint.as_deref().unwrap_or("any");
                let desc = p
                    .description
                    .as_ref()
                    .map(|d| format!(" - {}", d))
                    .unwrap_or_default();
                format!("{}: {}{}", p.name, type_hint, desc)
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generates an example function call with realistic parameter values.
    fn generate_example_call(params: &[ParameterDoc]) -> String {
        if params.is_empty() {
            return "".to_string();
        }

        params
            .iter()
            .map(|p| {
                let example_value = match p.type_hint.as_deref() {
                    Some(t) if t.contains("string") || t.contains("String") => {
                        format!("'{}'", p.name)
                    }
                    Some(t) if t.contains("number") || t.contains("Number") => "42".to_string(),
                    Some(t) if t.contains("boolean") || t.contains("Boolean") => {
                        "true".to_string()
                    }
                    Some(t) if t.contains("[]") || t.contains("Array") => "[]".to_string(),
                    Some(t) if t.contains("{}") || t.contains("object") => "{}".to_string(),
                    _ => format!("{}", p.name),
                };
                example_value
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;

    #[test]
    fn test_format_parameters() {
        let params = vec![
            ParameterDoc {
                name: "name".to_string(),
                type_hint: Some("string".to_string()),
                description: Some("User name".to_string()),
            },
            ParameterDoc {
                name: "age".to_string(),
                type_hint: Some("number".to_string()),
                description: None,
            },
        ];

        let formatted = PromptBuilder::format_parameters(&params);
        assert!(formatted.contains("name: string - User name"));
        assert!(formatted.contains("age: number"));
    }

    #[test]
    fn test_generate_example_call() {
        let params = vec![
            ParameterDoc {
                name: "name".to_string(),
                type_hint: Some("string".to_string()),
                description: None,
            },
            ParameterDoc {
                name: "count".to_string(),
                type_hint: Some("number".to_string()),
                description: None,
            },
        ];

        let call = PromptBuilder::generate_example_call(&params);
        assert!(call.contains("'name'"));
        assert!(call.contains("42"));
    }

    #[test]
    fn test_build_function_prompt() {
        let symbol = ExportedSymbol {
            name: "calculateTotal".to_string(),
            kind: SymbolKind::Function,
            summary: Some("Calculates total".to_string()),
            parameters: vec![ParameterDoc {
                name: "items".to_string(),
                type_hint: Some("number[]".to_string()),
                description: Some("Array of numbers".to_string()),
            }],
            returns: Some("number".to_string()),
            deprecated: None,
            examples: vec![],
            tags: vec![],
            location: SourceLocation::new(1, 1),
        };

        let context = EnhancementContext {
            file_path: "src/math.ts".to_string(),
            surrounding_code: None,
        };

        let prompt = PromptBuilder::build_prompt(&symbol, &context);

        assert!(prompt.contains("calculateTotal"));
        assert!(prompt.contains("items: number[]"));
        assert!(prompt.contains("number"));
        assert!(prompt.contains("src/math.ts"));
        assert!(prompt.contains("JSON"));
    }
}
