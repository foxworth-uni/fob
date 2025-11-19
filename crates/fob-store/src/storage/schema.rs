//! SurrealDB schema definitions for module graph storage.

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::storage::StorageError;

/// Define the database schema for module graph storage.
pub async fn define_schema(db: &Surreal<Any>) -> Result<(), StorageError> {
    // Define module table
    let module_schema = r#"
        DEFINE TABLE module SCHEMAFULL;
        DEFINE FIELD id ON module TYPE string ASSERT $value != NONE;
        DEFINE FIELD path ON module TYPE string;
        DEFINE FIELD source_type ON module TYPE string;
        DEFINE FIELD imports ON module TYPE array;
        DEFINE FIELD exports ON module TYPE array;
        DEFINE FIELD has_side_effects ON module TYPE bool DEFAULT false;
        DEFINE FIELD is_entry ON module TYPE bool DEFAULT false;
        DEFINE FIELD is_external ON module TYPE bool DEFAULT false;
        DEFINE FIELD original_size ON module TYPE number DEFAULT 0;
        DEFINE FIELD bundled_size ON module TYPE option<number>;
        DEFINE FIELD symbol_table ON module TYPE string;
        DEFINE INDEX idx_module_id ON module FIELDS id UNIQUE;
        DEFINE INDEX idx_module_path ON module FIELDS path;
    "#;

    db.query(module_schema)
        .await
        .map_err(|e| StorageError::Schema(e.to_string()))?;

    // Define dependency relation table
    let dependency_schema = r#"
        DEFINE TABLE depends_on TYPE RELATION FROM module TO module;
        DEFINE FIELD from ON depends_on TYPE record<module>;
        DEFINE FIELD to ON depends_on TYPE record<module>;
        DEFINE INDEX idx_depends_from ON depends_on FIELDS from;
        DEFINE INDEX idx_depends_to ON depends_on FIELDS to;
    "#;

    db.query(dependency_schema)
        .await
        .map_err(|e| StorageError::Schema(e.to_string()))?;

    // Define external dependency table
    let external_schema = r#"
        DEFINE TABLE external_dep SCHEMAFULL;
        DEFINE FIELD specifier ON external_dep TYPE string;
        DEFINE FIELD importers ON external_dep TYPE array<string>;
        DEFINE INDEX idx_external_specifier ON external_dep FIELDS specifier UNIQUE;
    "#;

    db.query(external_schema)
        .await
        .map_err(|e| StorageError::Schema(e.to_string()))?;

    Ok(())
}

