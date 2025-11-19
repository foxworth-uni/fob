//! SurrealDB-backed storage for module graph.
//!
//! This module provides persistent graph storage using SurrealDB with platform-specific
//! backends: kv-mem for WASM/browser, kv-rocksdb for native.

pub mod queries;
pub mod schema;

mod impl_queries;

use std::path::PathBuf;
use std::sync::Arc;

use surrealdb::engine::any::Any;
use surrealdb::Surreal;
use thiserror::Error;

pub use queries::GraphQueries;

/// Storage backend for module graph.
#[derive(Debug)]
pub struct GraphStorage {
    db: Arc<Surreal<Any>>,
    namespace: String,
    database: String,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Database connection failed: {0}")]
    Connection(String),
    #[error("Query execution failed: {0}")]
    Query(String),
    #[error("Schema initialization failed: {0}")]
    Schema(String),
}

impl GraphStorage {
    /// Create a new graph storage instance with platform-specific backend.
    ///
    /// - WASM: Uses in-memory kv-mem backend
    /// - Native: Uses RocksDB backend with persistent storage
    pub async fn new() -> std::result::Result<Self, StorageError> {
        Self::with_path(None).await
    }

    /// Create storage with a specific database path (native only).
    pub async fn with_path(path: Option<PathBuf>) -> std::result::Result<Self, StorageError> {
        let db = Self::connect(path).await?;
        let storage = Self {
            db: Arc::new(db),
            namespace: "fob".to_string(),
            database: "graph".to_string(),
        };

        // Initialize schema
        storage.init_schema().await?;

        Ok(storage)
    }

    async fn connect(path: Option<PathBuf>) -> std::result::Result<Surreal<Any>, StorageError> {
        #[cfg(target_family = "wasm")]
        {
            use surrealdb::engine::any::connect;
            let db = connect("mem://")
                .await
                .map_err(|e| StorageError::Connection(e.to_string()))?;
            Ok(db)
        }

        #[cfg(not(target_family = "wasm"))]
        {
            use surrealdb::engine::any::connect;
            let db_path = path
                .unwrap_or_else(|| PathBuf::from("./.fob-cache/graph.db"))
                .to_string_lossy()
                .to_string();
            let endpoint = format!("rocksdb:{db_path}");
            let db = connect(&endpoint)
                .await
                .map_err(|e| StorageError::Connection(e.to_string()))?;
            Ok(db)
        }
    }

    async fn init_schema(&self) -> std::result::Result<(), StorageError> {
        self.db
            .use_ns(&self.namespace)
            .use_db(&self.database)
            .await
            .map_err(|e| StorageError::Schema(e.to_string()))?;

        schema::define_schema(&self.db).await?;

        Ok(())
    }

    /// Get a reference to the underlying database.
    pub fn db(&self) -> &Surreal<Any> {
        &self.db
    }

    /// Get the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Get the database name.
    pub fn database(&self) -> &str {
        &self.database
    }
}

// GraphQueries implementation is in impl_queries.rs
