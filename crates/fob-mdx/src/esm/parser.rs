//! ESM statement parsing and classification

/// Checks if an ESM block contains named exports
///
/// # Examples
/// - `export const meta = {...}` → true
/// - `export { Comp }` → true
/// - `export default function` → false
pub fn has_named_exports(code: &str) -> bool {
    code.contains("export ")
        && !code.trim_start().starts_with("export default")
        && (code.contains("export const ")
            || code.contains("export let ")
            || code.contains("export var ")
            || code.contains("export function ")
            || code.contains("export class ")
            || code.contains("export {"))
}

/// Checks if an ESM block is a re-export
///
/// # Examples
/// - `export { Comp } from './comp'` → true
/// - `export * from './utils'` → true
/// - `export { default as Comp } from './comp'` → true
pub fn is_reexport(code: &str) -> bool {
    code.contains("export ") && code.contains(" from ")
}

/// Extracts the default export name if present
///
/// # Examples
/// - `export default function MDXContent` → Some("MDXContent")
/// - `export default Comp` → Some("Comp")
/// - `export const x = 1` → None
pub fn get_default_export_name(code: &str) -> Option<String> {
    let trimmed = code.trim();

    let after_default = trimmed.strip_prefix("export default ")?;

    // Try to extract function name
    if let Some(after_fn) = after_default.strip_prefix("function ") {
        if let Some(end) = after_fn.find(|c: char| c == '(' || c.is_whitespace()) {
            return Some(after_fn[..end].trim().to_string());
        }
    }

    // Try to extract class name
    if let Some(after_class) = after_default.strip_prefix("class ") {
        if let Some(end) = after_class.find(|c: char| c == '{' || c.is_whitespace()) {
            return Some(after_class[..end].trim().to_string());
        }
    }

    // Try to extract variable/identifier
    let end = after_default
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(after_default.len());

    let name = after_default[..end].trim();
    if !name.is_empty() {
        return Some(name.to_string());
    }

    None
}

/// Extracts imported component names from an import statement
///
/// Returns all identifiers that are imported and can be used as JSX components.
/// This includes default imports, named imports, and aliased imports (using the alias).
///
/// # Examples
/// - `import Button from './button'` → `["Button"]`
/// - `import { Button } from './ui'` → `["Button"]`
/// - `import { Button, Card } from './ui'` → `["Button", "Card"]`
/// - `import { Button as Btn } from './ui'` → `["Btn"]`
/// - `import React, { useState } from 'react'` → `["React"]`
/// - `import * as UI from './ui'` → `["UI"]`
pub fn extract_imported_names(code: &str) -> Vec<String> {
    let mut names = Vec::new();
    let trimmed = code.trim();

    // Must be an import statement
    if !trimmed.starts_with("import ") {
        return names;
    }

    // Remove "import " prefix and find " from " separator
    let after_import = &trimmed[7..]; // Skip "import "
    let Some(from_pos) = after_import.find(" from ") else {
        return names;
    };

    let import_clause = after_import[..from_pos].trim();

    // Handle namespace import: import * as Name from '...'
    if let Some(as_pos) = import_clause.find(" as ") {
        if import_clause[..as_pos].trim() == "*" {
            let namespace_name = import_clause[as_pos + 4..].trim();
            if !namespace_name.is_empty() {
                names.push(namespace_name.to_string());
            }
            return names;
        }
    }

    // Check if there are named imports (braces)
    if let Some(brace_start) = import_clause.find('{') {
        if let Some(brace_end) = import_clause.find('}') {
            // Extract content between braces
            let named_imports = &import_clause[brace_start + 1..brace_end];

            // Parse each named import
            for item in named_imports.split(',') {
                let item = item.trim();
                // Handle aliased import: "Button as Btn"
                if let Some(as_pos) = item.find(" as ") {
                    let alias = item[as_pos + 4..].trim();
                    if !alias.is_empty() && is_component_name(alias) {
                        names.push(alias.to_string());
                    }
                } else if !item.is_empty() && is_component_name(item) {
                    names.push(item.to_string());
                }
            }

            // Check for default import before the braces
            let before_brace = import_clause[..brace_start].trim().trim_end_matches(',');
            if !before_brace.is_empty() && is_component_name(before_brace) {
                names.insert(0, before_brace.to_string());
            }
        }
    } else {
        // No braces - could be default import or namespace import
        let trimmed = import_clause.trim();
        if !trimmed.is_empty() && is_component_name(trimmed) {
            names.push(trimmed.to_string());
        }
    }

    names
}

/// Check if a name could be a component (starts with uppercase)
fn is_component_name(name: &str) -> bool {
    name.chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_named_exports() {
        assert!(has_named_exports("export const x = 1"));
        assert!(has_named_exports("export { foo }"));
        assert!(!has_named_exports("export default foo"));
        assert!(!has_named_exports("import x from 'y'"));
    }

    #[test]
    fn test_is_reexport() {
        assert!(is_reexport("export { foo } from './bar'"));
        assert!(is_reexport("export * from './utils'"));
        assert!(!is_reexport("export const x = 1"));
    }

    #[test]
    fn test_get_default_export_name() {
        assert_eq!(
            get_default_export_name("export default function Foo() {}"),
            Some("Foo".to_string())
        );
        assert_eq!(
            get_default_export_name("export default Bar"),
            Some("Bar".to_string())
        );
        assert_eq!(get_default_export_name("export const x = 1"), None);
    }

    #[test]
    fn test_extract_imported_names() {
        // Default import
        assert_eq!(
            extract_imported_names("import Button from './button'"),
            vec!["Button"]
        );

        // Named import (single)
        assert_eq!(
            extract_imported_names("import { Button } from './ui'"),
            vec!["Button"]
        );

        // Named imports (multiple)
        assert_eq!(
            extract_imported_names("import { Button, Card } from './ui'"),
            vec!["Button", "Card"]
        );

        // Aliased import
        assert_eq!(
            extract_imported_names("import { Button as Btn } from './ui'"),
            vec!["Btn"]
        );

        // Mixed default + named
        assert_eq!(
            extract_imported_names("import React, { useState } from 'react'"),
            vec!["React"]
        );

        // Namespace import
        assert_eq!(
            extract_imported_names("import * as UI from './ui'"),
            vec!["UI"]
        );

        // Lowercase imports should be filtered out
        assert_eq!(
            extract_imported_names("import { useState, useEffect } from 'react'"),
            Vec::<String>::new()
        );

        // Mixed case
        assert_eq!(
            extract_imported_names("import { Button, useState, Card } from './ui'"),
            vec!["Button", "Card"]
        );

        // Not an import statement
        assert_eq!(
            extract_imported_names("export const x = 1"),
            Vec::<String>::new()
        );
    }
}
