#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct MdxSyntaxError {
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct MissingExportError {
    pub export_name: String,
    pub module_id: String,
    pub available_exports: Vec<String>,
    pub suggestion: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct TransformDiagnostic {
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub severity: String, // 'error' | 'warning'
    pub help: Option<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct TransformError {
    pub path: String,
    pub diagnostics: Vec<TransformDiagnostic>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct CircularDependencyError {
    pub cycle_path: Vec<String>,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct InvalidEntryError {
    pub path: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct NoEntriesError {}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct PluginError {
    pub name: String,
    pub message: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct RuntimeError {
    pub message: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct ValidationError {
    pub message: String,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub struct MultipleDiagnostics {
    pub errors: Vec<FobErrorDetails>,
    pub primary_message: String,
}

/// Versioned error envelope for API stability
#[derive(serde::Serialize)]
pub struct ErrorEnvelope {
    /// Error format version (incremented when error structure changes)
    pub version: u32,
    /// The actual error details
    pub error: FobErrorDetails,
}

#[derive(serde::Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum FobErrorDetails {
    MdxSyntax(MdxSyntaxError),
    MissingExport(MissingExportError),
    Transform(TransformError),
    CircularDependency(CircularDependencyError),
    InvalidEntry(InvalidEntryError),
    NoEntries(NoEntriesError),
    Plugin(PluginError),
    Runtime(RuntimeError),
    Validation(ValidationError),
    Multiple(MultipleDiagnostics),
}

/// Unified NAPI error format for JavaScript consumption
#[derive(serde::Serialize)]
pub struct NapiError {
    pub kind: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub help: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl FobErrorDetails {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"type\":\"runtime\",\"message\":\"Failed to serialize error\"}".to_string()
        })
    }

    /// Convert to unified NAPI error format
    pub fn to_napi_error(&self) -> NapiError {
        match self {
            FobErrorDetails::MdxSyntax(err) => NapiError {
                kind: "MdxSyntax".to_string(),
                message: err.message.clone(),
                file: err.file.clone(),
                line: err.line,
                column: err.column,
                help: err.suggestion.clone(),
                details: None,
            },
            FobErrorDetails::Transform(err) => {
                let first_diag = err.diagnostics.first();
                NapiError {
                    kind: "Transform".to_string(),
                    message: first_diag
                        .map(|d| d.message.clone())
                        .unwrap_or_else(|| "Transform error".to_string()),
                    file: Some(err.path.clone()),
                    line: first_diag.map(|d| d.line),
                    column: first_diag.map(|d| d.column),
                    help: first_diag.and_then(|d| d.help.clone()),
                    details: Some(serde_json::json!({
                        "diagnostics": err.diagnostics,
                        "path": err.path
                    })),
                }
            }
            FobErrorDetails::MissingExport(err) => NapiError {
                kind: "MissingExport".to_string(),
                message: format!(
                    "Missing export '{}' from module '{}'",
                    err.export_name, err.module_id
                ),
                file: Some(err.module_id.clone()),
                line: None,
                column: None,
                help: err.suggestion.clone(),
                details: Some(serde_json::json!({
                    "export_name": err.export_name,
                    "module_id": err.module_id,
                    "available_exports": err.available_exports
                })),
            },
            FobErrorDetails::CircularDependency(err) => NapiError {
                kind: "CircularDependency".to_string(),
                message: format!("Circular dependency detected: {}", err.cycle_path.join(" -> ")),
                file: err.cycle_path.first().cloned(),
                line: None,
                column: None,
                help: Some("Break the circular dependency by refactoring your imports".to_string()),
                details: Some(serde_json::json!({
                    "cycle_path": err.cycle_path
                })),
            },
            FobErrorDetails::InvalidEntry(err) => NapiError {
                kind: "InvalidEntry".to_string(),
                message: format!("Invalid entry point: {}", err.path),
                file: Some(err.path.clone()),
                line: None,
                column: None,
                help: Some("Ensure the entry file exists and is accessible".to_string()),
                details: None,
            },
            FobErrorDetails::NoEntries(_) => NapiError {
                kind: "NoEntries".to_string(),
                message: "No entry points configured".to_string(),
                file: None,
                line: None,
                column: None,
                help: Some("Provide at least one entry point in your configuration".to_string()),
                details: None,
            },
            FobErrorDetails::Plugin(err) => NapiError {
                kind: "Plugin".to_string(),
                message: format!("Plugin '{}' error: {}", err.name, err.message),
                file: None,
                line: None,
                column: None,
                help: None,
                details: Some(serde_json::json!({
                    "plugin_name": err.name,
                    "message": err.message
                })),
            },
            FobErrorDetails::Runtime(err) => NapiError {
                kind: "Runtime".to_string(),
                message: err.message.clone(),
                file: None,
                line: None,
                column: None,
                help: None,
                details: None,
            },
            FobErrorDetails::Validation(err) => NapiError {
                kind: "Validation".to_string(),
                message: err.message.clone(),
                file: None,
                line: None,
                column: None,
                help: Some("Check your configuration for errors".to_string()),
                details: None,
            },
            FobErrorDetails::Multiple(err) => {
                let primary = err.errors.first();
                NapiError {
                    kind: "Multiple".to_string(),
                    message: err.primary_message.clone(),
                    file: primary.and_then(|e| e.to_napi_error().file),
                    line: primary.and_then(|e| e.to_napi_error().line),
                    column: primary.and_then(|e| e.to_napi_error().column),
                    help: None,
                    details: Some(serde_json::json!({
                        "errors": err.errors.iter().map(|e| e.to_napi_error()).collect::<Vec<_>>(),
                        "count": err.errors.len()
                    })),
                }
            }
        }
    }

    /// Serialize to JSON string using NAPI format
    pub fn to_napi_json_string(&self) -> String {
        serde_json::to_string(&self.to_napi_error()).unwrap_or_else(|_| {
            serde_json::json!({
                "kind": "Runtime",
                "message": "Failed to serialize error"
            })
            .to_string()
        })
    }

    /// Wrap error in versioned envelope
    pub fn into_envelope(self, version: u32) -> ErrorEnvelope {
        ErrorEnvelope {
            version,
            error: self,
        }
    }

    /// Create versioned envelope with current version (1)
    pub fn into_envelope_v1(self) -> ErrorEnvelope {
        self.into_envelope(1)
    }
}

// Miette diagnostic support (for CLI usage)
#[cfg(not(target_family = "wasm"))]
mod miette;

#[cfg(not(target_family = "wasm"))]
pub use miette::to_miette_diagnostic;
