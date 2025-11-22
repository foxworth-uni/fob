//! Type definitions for the in-memory ModuleGraph implementation.

use super::super::{ImportSpecifier, ModuleId, SourceSpan};

/// Information about a class member symbol
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClassMemberInfo {
    pub module_id: ModuleId,
    pub symbol: super::super::symbol::Symbol,
    pub metadata: super::super::symbol::ClassMemberMetadata,
}

/// Information about an enum member
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnumMemberInfo {
    pub module_id: ModuleId,
    pub symbol: super::super::symbol::Symbol,
    pub value: Option<super::super::symbol::EnumMemberValue>,
}

/// A side-effect-only import (no bindings, just execution)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SideEffectImport {
    pub importer: ModuleId,
    pub source: String,
    pub resolved_to: Option<ModuleId>,
    pub span: SourceSpan,
}

/// Information about a namespace import
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamespaceImportInfo {
    pub importer: ModuleId,
    pub namespace_name: String,
    pub source: String,
    pub resolved_to: Option<ModuleId>,
}

/// A type-only import (TypeScript)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TypeOnlyImport {
    pub importer: ModuleId,
    pub source: String,
    pub specifiers: Vec<ImportSpecifier>,
    pub span: SourceSpan,
}
