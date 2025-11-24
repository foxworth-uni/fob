//! Metadata types for specialized symbol kinds.

use serde::{Deserialize, Serialize};

/// Visibility modifier for class members
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Visibility {
    /// public (default in JS, explicit in TS)
    Public,
    /// private (TS or JS private fields with #)
    Private,
    /// protected (TS only)
    Protected,
}

/// Additional metadata for class member symbols
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassMemberMetadata {
    /// Visibility of the member
    pub visibility: Visibility,
    /// Whether the member is static
    pub is_static: bool,
    /// The class this member belongs to
    pub class_name: String,
    /// Whether this is an accessor (getter/setter)
    pub is_accessor: bool,
    /// Whether this member is abstract (TS only)
    pub is_abstract: bool,
    /// Whether this member is readonly (TS only)
    pub is_readonly: bool,
}

impl ClassMemberMetadata {
    /// Create metadata for a class member
    pub fn new(visibility: Visibility, is_static: bool, class_name: String) -> Self {
        Self {
            visibility,
            is_static,
            class_name,
            is_accessor: false,
            is_abstract: false,
            is_readonly: false,
        }
    }
}

/// Enum member value types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnumMemberValue {
    /// Numeric value
    Number(i64),
    /// String value
    String(String),
    /// Computed value (not statically known)
    Computed,
}

/// Metadata for enum member tracking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnumMemberMetadata {
    /// The enum this member belongs to
    pub enum_name: String,
    /// The value of the enum member (if constant)
    pub value: Option<EnumMemberValue>,
}

/// Code quality metrics for functions and classes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeQualityMetadata {
    /// Number of lines in the function/class (approximate)
    pub line_count: Option<usize>,
    /// Number of parameters (for functions)
    pub parameter_count: Option<usize>,
    /// Cyclomatic complexity (for functions)
    pub complexity: Option<usize>,
    /// Maximum nesting depth
    pub max_nesting_depth: Option<usize>,
    /// Number of return statements (for functions)
    pub return_count: Option<usize>,
    /// Number of methods (for classes)
    pub method_count: Option<usize>,
    /// Number of fields/properties (for classes)
    pub field_count: Option<usize>,
}

impl CodeQualityMetadata {
    /// Create new code quality metadata with all fields optional
    pub fn new() -> Self {
        Self {
            line_count: None,
            parameter_count: None,
            complexity: None,
            max_nesting_depth: None,
            return_count: None,
            method_count: None,
            field_count: None,
        }
    }

    /// Create metadata for a function
    pub fn for_function(
        line_count: Option<usize>,
        parameter_count: Option<usize>,
        complexity: Option<usize>,
        max_nesting_depth: Option<usize>,
        return_count: Option<usize>,
    ) -> Self {
        Self {
            line_count,
            parameter_count,
            complexity,
            max_nesting_depth,
            return_count,
            method_count: None,
            field_count: None,
        }
    }

    /// Create metadata for a class
    pub fn for_class(
        line_count: Option<usize>,
        method_count: Option<usize>,
        field_count: Option<usize>,
    ) -> Self {
        Self {
            line_count,
            parameter_count: None,
            complexity: None,
            max_nesting_depth: None,
            return_count: None,
            method_count,
            field_count,
        }
    }
}

impl Default for CodeQualityMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol metadata (extensible for different symbol kinds)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SymbolMetadata {
    /// No additional metadata
    #[default]
    None,
    /// Class member metadata
    ClassMember(ClassMemberMetadata),
    /// Enum member metadata
    EnumMember(EnumMemberMetadata),
    /// Code quality metrics (for functions and classes)
    CodeQuality(CodeQualityMetadata),
}

