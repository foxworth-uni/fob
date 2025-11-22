#[derive(serde::Serialize)]
pub struct MdxSyntaxError {
    pub r#type: String,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub context: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(serde::Serialize)]
pub struct MissingExportError {
    pub r#type: String,
    pub export_name: String,
    pub module_id: String,
    pub available_exports: Vec<String>,
    pub suggestion: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TransformDiagnostic {
    pub message: String,
    pub line: u32,
    pub column: u32,
    pub severity: String, // 'error' | 'warning'
    pub help: Option<String>,
}

#[derive(serde::Serialize)]
pub struct TransformError {
    pub r#type: String,
    pub path: String,
    pub diagnostics: Vec<TransformDiagnostic>,
}

#[derive(serde::Serialize)]
pub struct CircularDependencyError {
    pub r#type: String,
    pub cycle_path: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct InvalidEntryError {
    pub r#type: String,
    pub path: String,
}

#[derive(serde::Serialize)]
pub struct NoEntriesError {
    pub r#type: String,
}

#[derive(serde::Serialize)]
pub struct PluginError {
    pub r#type: String,
    pub name: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct RuntimeError {
    pub r#type: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct ValidationError {
    pub r#type: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct MultipleDiagnostics {
    pub r#type: String, // "multiple"
    pub errors: Vec<FobErrorDetails>,
    pub primary_message: String,
}

#[derive(serde::Serialize)]
#[serde(untagged)] // This might conflict with manual type fields, let's check
                   // The structs already have a `type` field. Serde tag might duplicate it or overwrite it.
                   // Actually, the TypeScript definition uses a discriminated union where `type` is a literal string.
                   // My structs have `pub r#type: String`.
                   // If I use `#[serde(untagged)]`, it will just serialize the struct fields.
                   // Since the structs HAVE the type field, untagged is fine.
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

impl FobErrorDetails {
    pub fn to_json_string(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            "{\"type\":\"runtime\",\"message\":\"Failed to serialize error\"}".to_string()
        })
    }
}
