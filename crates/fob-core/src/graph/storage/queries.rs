//! Graph query implementations for SurrealDB storage.

use surrealdb::engine::any::Any;
use surrealdb::Surreal;

use crate::graph::{ExternalDependency, Module, ModuleId};
use crate::graph::storage::StorageError;

/// Trait for graph query operations.
pub trait GraphQueries {
    fn db(&self) -> &Surreal<Any>;
    fn namespace(&self) -> &str;
    fn database(&self) -> &str;
    
    /// Ensure database context is set.
    async fn ensure_context(&self) -> std::result::Result<(), StorageError>;
    
    /// Store a module in the database.
    async fn store_module(&self, module: &Module) -> std::result::Result<(), StorageError>;
    
    /// Retrieve a module by ID.
    async fn get_module(&self, id: &ModuleId) -> std::result::Result<Option<Module>, StorageError>;
    
    /// Get all modules.
    async fn get_all_modules(&self) -> std::result::Result<Vec<Module>, StorageError>;
    
    /// Add a dependency edge.
    async fn add_dependency(&self, from: &ModuleId, to: &ModuleId) -> std::result::Result<(), StorageError>;
    
    /// Get dependencies of a module.
    async fn get_dependencies(&self, id: &ModuleId) -> std::result::Result<Vec<ModuleId>, StorageError>;
    
    /// Get dependents (reverse dependencies) of a module.
    async fn get_dependents(&self, id: &ModuleId) -> std::result::Result<Vec<ModuleId>, StorageError>;
    
    /// Get all entry points.
    async fn get_entry_points(&self) -> std::result::Result<Vec<ModuleId>, StorageError>;
    
    /// Store an external dependency.
    async fn store_external_dependency(&self, dep: &ExternalDependency) -> std::result::Result<(), StorageError>;
    
    /// Get all external dependencies.
    async fn get_external_dependencies(&self) -> std::result::Result<Vec<ExternalDependency>, StorageError>;
    
    /// Clear all graph data.
    async fn clear_all(&self) -> std::result::Result<(), StorageError>;
    
    /// Helper to convert database record to Module.
    fn module_from_record(&self, record: ModuleRecord) -> std::result::Result<Module, StorageError>;
}

// Database record types - exported for use in implementations
#[derive(serde::Deserialize)]
pub struct ModuleRecord {
    pub id: String,
    pub path: String,
    pub source_type: String,
    pub imports: String, // JSON string
    pub exports: String, // JSON string
    pub has_side_effects: bool,
    pub is_entry: bool,
    pub is_external: bool,
    pub original_size: i64,
    pub bundled_size: Option<i64>,
    pub symbol_table: String, // JSON string
}

#[derive(serde::Deserialize)]
pub(super) struct EntryPointRecord {
    pub id: String,
}

#[derive(serde::Deserialize)]
pub(super) struct ExternalDepRecord {
    pub specifier: String,
    pub importers: Vec<String>,
}
