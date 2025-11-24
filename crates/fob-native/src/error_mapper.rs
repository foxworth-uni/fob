use crate::error::{MdxSyntaxError, PluginError, *};
use fob_bundler::diagnostics::{DiagnosticContext, DiagnosticKind, ExtractedDiagnostic};
use fob_bundler::Error as BundlerError;

/// Map fob-bundler errors to structured FobErrorDetails
pub fn map_bundler_error(error: &BundlerError) -> FobErrorDetails {
    match error {
        // Direct mappings for fob-bundler error variants
        BundlerError::InvalidConfig(msg) => {
            // Special case: empty entries should map to NoEntries
            if msg.contains("No entries provided") || msg.contains("No entry") {
                FobErrorDetails::NoEntries(NoEntriesError {})
            } else {
                FobErrorDetails::Validation(ValidationError {
                    message: msg.clone(),
                })
            }
        },

        BundlerError::InvalidOutputPath(path) => FobErrorDetails::InvalidEntry(InvalidEntryError {
            path: path.clone(),
        }),

        BundlerError::WriteFailure(msg) => FobErrorDetails::Runtime(RuntimeError {
            message: format!("Write failure: {}", msg),
        }),

        BundlerError::OutputExists(msg) => FobErrorDetails::Runtime(RuntimeError {
            message: format!("Output exists: {}", msg),
        }),

        // Rolldown bundler errors - now use extracted diagnostics
        BundlerError::Bundler(diagnostics) => {
            if diagnostics.is_empty() {
                FobErrorDetails::Runtime(RuntimeError {
                    message: "Unknown bundler error".to_string(),
                })
            } else if diagnostics.len() == 1 {
                map_single_diagnostic(&diagnostics[0])
            } else {
                map_multiple_diagnostics(diagnostics)
            }
        }

        // I/O and other errors
        BundlerError::Io(e) => FobErrorDetails::Runtime(RuntimeError {
            message: format!("I/O error: {}", e),
        }),

        BundlerError::IoError { message, .. } => FobErrorDetails::Runtime(RuntimeError {
            message: message.clone(),
        }),

        BundlerError::AssetNotFound {
            specifier,
            searched_from,
        } => FobErrorDetails::Runtime(RuntimeError {
            message: format!(
                "Asset not found: {} (searched from: {})",
                specifier, searched_from
            ),
        }),

        BundlerError::AssetSecurityViolation { path, reason } => {
            FobErrorDetails::Runtime(RuntimeError {
                message: format!("Asset security violation: {} - {}", path, reason),
            })
        }

        BundlerError::AssetTooLarge {
            path,
            size,
            max_size,
        } => FobErrorDetails::Runtime(RuntimeError {
            message: format!(
                "Asset too large: {} ({} bytes exceeds limit of {} bytes)",
                path, size, max_size
            ),
        }),

        BundlerError::Foundation(e) => FobErrorDetails::Runtime(RuntimeError {
            message: format!("Foundation error: {}", e),
        }),
    }
}

/// Map a single diagnostic to FobErrorDetails
fn map_single_diagnostic(diag: &ExtractedDiagnostic) -> FobErrorDetails {
    match &diag.kind {
        DiagnosticKind::MissingExport => {
            // Use structured context if available, otherwise fall back to parsing
            let (export_name, module_id, available_exports) = if let Some(DiagnosticContext::MissingExport { export_name, module_id, available_exports }) = &diag.context {
                (export_name.clone(), module_id.clone(), available_exports.clone())
            } else {
                extract_missing_export_info(&diag.message, &diag.help)
            };

            FobErrorDetails::MissingExport(MissingExportError {
                export_name,
                module_id,
                available_exports,
                suggestion: diag.help.clone(),
            })
        }

        DiagnosticKind::ParseError | DiagnosticKind::Transform => {
            // Check if this is an MDX file - if so, use MdxSyntaxError
            let is_mdx = diag
                .file
                .as_ref()
                .map(|f| f.ends_with(".mdx") || f.ends_with(".md"))
                .unwrap_or(false);

            if is_mdx {
                FobErrorDetails::MdxSyntax(MdxSyntaxError {
                    message: diag.message.clone(),
                    file: diag.file.clone(),
                    line: diag.line,
                    column: diag.column,
                    context: None, // Could be extracted from help text if needed
                    suggestion: diag.help.clone(),
                })
            } else {
                // Create TransformError with diagnostics array
                let path = diag.file.clone().unwrap_or_else(|| "unknown".to_string());

                let diagnostic = TransformDiagnostic {
                    message: diag.message.clone(),
                    line: diag.line.unwrap_or(0),
                    column: diag.column.unwrap_or(0),
                    severity: match diag.severity {
                        fob_bundler::diagnostics::DiagnosticSeverity::Error => "error".to_string(),
                        fob_bundler::diagnostics::DiagnosticSeverity::Warning => {
                            "warning".to_string()
                        }
                    },
                    help: diag.help.clone(),
                };

                FobErrorDetails::Transform(TransformError {
                    path,
                    diagnostics: vec![diagnostic],
                })
            }
        }

        DiagnosticKind::Plugin => {
            // Use structured context if available, otherwise fall back to parsing
            let (name, message) = if let Some(DiagnosticContext::Plugin { plugin_name }) = &diag.context {
                (plugin_name.clone(), diag.message.clone())
            } else {
                extract_plugin_info(&diag.message)
            };

            FobErrorDetails::Plugin(PluginError {
                name,
                message,
            })
        }

        DiagnosticKind::CircularDependency => {
            // Use structured context if available, otherwise fall back to parsing
            let cycle_path = if let Some(DiagnosticContext::CircularDependency { cycle_path }) = &diag.context {
                cycle_path.clone()
            } else {
                extract_cycle_path(&diag.message)
            };

            FobErrorDetails::CircularDependency(CircularDependencyError {
                cycle_path,
            })
        }

        DiagnosticKind::UnresolvedEntry => {
            let path = if let Some(DiagnosticContext::UnresolvedEntry { entry_path }) = &diag.context {
                entry_path.clone()
            } else {
                diag.file.clone().unwrap_or_else(|| {
                    // Try to extract from message
                    extract_path_from_message(&diag.message).unwrap_or_else(|| "unknown".to_string())
                })
            };

            FobErrorDetails::InvalidEntry(InvalidEntryError {
                path,
            })
        }

        DiagnosticKind::UnresolvedImport
        | DiagnosticKind::InvalidOption
        | DiagnosticKind::Other(_) => {
            // Map to RuntimeError with context
            FobErrorDetails::Runtime(RuntimeError {
                message: format!("{}: {}", diag.kind, diag.message),
            })
        }
    }
}

/// Map multiple diagnostics to FobErrorDetails::Multiple
fn map_multiple_diagnostics(diagnostics: &[ExtractedDiagnostic]) -> FobErrorDetails {
    let errors: Vec<FobErrorDetails> = diagnostics.iter().map(map_single_diagnostic).collect();

    // Create primary message from first diagnostic
    let primary_message = if let Some(first) = diagnostics.first() {
        format!("{}: {}", first.kind, first.message)
    } else {
        "Multiple bundler errors".to_string()
    };

    FobErrorDetails::Multiple(MultipleDiagnostics {
        errors,
        primary_message,
    })
}

// Helper functions for extracting structured information from diagnostic messages

/// Extract missing export information from error message and help text
fn extract_missing_export_info(
    message: &str,
    help: &Option<String>,
) -> (String, String, Vec<String>) {
    // Try to extract export_name
    let export_name = extract_field(message, "export_name")
        .or_else(|| extract_quoted(message, "export"))
        .or_else(|| {
            // Look for patterns like "export 'Foo'" or "export `Foo`"
            if let Some(start) = message.find("export") {
                let after = &message[start + 6..];
                if let Some(quote_start) = after.find(['\'', '`', '"']) {
                    let after_quote = &after[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find(['\'', '`', '"']) {
                        return Some(after_quote[..quote_end].to_string());
                    }
                }
            }
            None
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Try to extract module_id
    let module_id = extract_field(message, "requested_module")
        .or_else(|| extract_field(message, "module"))
        .or_else(|| extract_path_from_message(message))
        .unwrap_or_else(|| "unknown".to_string());

    // Try to extract available_exports from help text
    let available_exports = help
        .as_ref()
        .and_then(|h| extract_available_exports(h))
        .unwrap_or_default();

    (export_name, module_id, available_exports)
}

/// Extract cycle path from circular dependency message
fn extract_cycle_path(message: &str) -> Vec<String> {
    // Look for patterns like "A -> B -> C" or "[A, B, C]"
    if let Some(start) = message.find("->") {
        // Split by "->" and clean up
        let cycle: Vec<String> = message[start..]
            .split("->")
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !cycle.is_empty() {
            return cycle;
        }
    }

    // Fallback: return the message as a single path element
    vec![message.to_string()]
}

/// Extract path from message
fn extract_path_from_message(text: &str) -> Option<String> {
    // Look for file paths (typically contain .js, .ts, .jsx, .tsx)
    for ext in &[".js", ".ts", ".jsx", ".tsx", ".mjs", ".cjs"] {
        if let Some(pos) = text.find(ext) {
            // Backtrack to find the start of the path
            let before = &text[..=pos + ext.len()];
            // Look for common path indicators
            for indicator in &["in ", "at ", "file: ", "path: ", "\"", "'"] {
                if let Some(start) = before.rfind(indicator) {
                    let path_start = start + indicator.len();
                    let path_str = &before[path_start..];
                    // Find end of path (space, newline, quote, comma)
                    if let Some(end) = path_str.find([' ', '\n', '"', '\'', ',']) {
                        return Some(path_str[..end].trim().to_string());
                    }
                    return Some(path_str.trim().to_string());
                }
            }
        }
    }
    None
}

/// Extract field value from text (e.g., "field: value")
fn extract_field(text: &str, field: &str) -> Option<String> {
    let pattern = format!("{}: ", field);
    if let Some(start) = text.find(&pattern) {
        let start = start + pattern.len();
        let rest = &text[start..];

        // Skip quotes if present
        let rest = rest.trim_start_matches('"');

        // Find end (comma, closing brace, or quote)
        if let Some(end) = rest.find(&[',', '}', '"'][..]) {
            return Some(rest[..end].trim().to_string());
        }
        return Some(rest.trim().to_string());
    }
    None
}

/// Extract quoted string near a keyword
fn extract_quoted(text: &str, near: &str) -> Option<String> {
    if let Some(pos) = text.find(near) {
        let after = &text[pos..];
        if let Some(start) = after.find('"') {
            let after_start = &after[start + 1..];
            if let Some(end) = after_start.find('"') {
                return Some(after_start[..end].to_string());
            }
        }
    }
    None
}

/// Extract available exports from help text
fn extract_available_exports(help: &str) -> Option<Vec<String>> {
    // Look for patterns like "Available: Foo, Bar, Baz" or "Available exports: ..."
    for pattern in &["Available: ", "Available exports: ", "Exports: "] {
        if let Some(pos) = help.find(pattern) {
            let after = &help[pos + pattern.len()..];
            // Take until newline or end
            let exports_str = after.lines().next().unwrap_or("").trim();
            if !exports_str.is_empty() {
                let exports: Vec<String> = exports_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !exports.is_empty() {
                    return Some(exports);
                }
            }
        }
    }
    None
}

/// Extract plugin name and message from error message
fn extract_plugin_info(message: &str) -> (String, String) {
    // Look for patterns like "Plugin 'name' failed: message" or "Plugin name: message"
    for pattern in &["Plugin '", "Plugin `", "Plugin \""] {
        if let Some(start) = message.find(pattern) {
            let after_quote = &message[start + pattern.len()..];
            if let Some(end) = after_quote.find(['\'', '`', '"']) {
                let name = after_quote[..end].to_string();
                // Find the message part (usually after ": " or " failed: ")
                let after_name = &after_quote[end + 1..];
                if let Some(msg_start) = after_name.find(": ") {
                    let msg = after_name[msg_start + 2..].trim().to_string();
                    if !msg.is_empty() {
                        return (name, msg);
                    }
                }
                // Fallback: use rest of message
                let msg = after_name.trim().to_string();
                if !msg.is_empty() {
                    return (name, msg);
                }
            }
        }
    }

    // Fallback: try to extract from "Plugin: name - message" pattern
    if let Some(pos) = message.find("Plugin: ") {
        let after = &message[pos + 8..];
        if let Some(dash_pos) = after.find(" - ") {
            let name = after[..dash_pos].trim().to_string();
            let msg = after[dash_pos + 3..].trim().to_string();
            if !name.is_empty() {
                return (name, msg);
            }
        }
    }

    // Last resort: use "unknown" as name and full message
    ("unknown".to_string(), message.to_string())
}
