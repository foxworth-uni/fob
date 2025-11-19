use serde::{Deserialize, Serialize};

use crate::graph::{ModuleId, SourceSpan};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransformationTrace {
    pub renames: Vec<RenameEvent>,
    pub imports: Vec<ImportResolution>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameEvent {
    pub module: ModuleId,
    pub phase: RenamePhase,
    pub original: String,
    pub renamed: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RenamePhase {
    ScopeHoisting,
    Deconflicting,
    Minification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResolution {
    pub module: ModuleId,
    pub specifier: String,
    pub local: Option<String>,
    pub renamed: Option<String>,
    pub outcome: ImportOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportOutcome {
    Internal(ModuleId),
    External(String),
    Missing,
}
