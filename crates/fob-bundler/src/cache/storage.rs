//! redb-backed cache storage.
//!
//! Provides persistent key-value storage using redb, an embedded ACID database.
//! The cache uses a single database file with ACID transactions for reliability.

use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use super::key::CacheKey;
use super::serialize::CachedBuild;

/// Cache table: maps cache keys to serialized builds.
const CACHE_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("cache");

/// Metadata table: stores cache-wide metadata.
const METADATA_TABLE: TableDefinition<&str, &str> = TableDefinition::new("metadata");

/// Error types for cache operations.
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    /// Cache entry not found.
    #[error("cache miss")]
    CacheMiss,

    /// Cache database error.
    #[error("cache database error: {0}")]
    DatabaseError(String),

    /// Serialization error.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// Deserialization error.
    #[error("deserialization error: {0}")]
    DeserializationError(String),

    /// IO error.
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    /// Cache version mismatch.
    #[error("cache version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    /// Cache corrupted.
    #[error("cache corrupted: {0}")]
    Corrupted(String),

    /// Invalid data in cache.
    #[error("invalid data: {0}")]
    InvalidData(String),
}

impl From<redb::Error> for CacheError {
    fn from(err: redb::Error) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<redb::DatabaseError> for CacheError {
    fn from(err: redb::DatabaseError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<redb::TableError> for CacheError {
    fn from(err: redb::TableError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<redb::TransactionError> for CacheError {
    fn from(err: redb::TransactionError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<redb::StorageError> for CacheError {
    fn from(err: redb::StorageError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

impl From<redb::CommitError> for CacheError {
    fn from(err: redb::CommitError) -> Self {
        CacheError::DatabaseError(err.to_string())
    }
}

/// Persistent cache store using redb.
pub struct CacheStore {
    db: Database,
}

impl CacheStore {
    /// Open or create a cache store at the given directory.
    ///
    /// Creates the directory and database file if they don't exist.
    /// The database file is stored at `<cache_dir>/cache.redb`.
    pub fn open(cache_dir: &Path) -> Result<Self, CacheError> {
        // Create cache directory if needed
        std::fs::create_dir_all(cache_dir)?;

        let db_path = cache_dir.join("cache.redb");

        // Open database with default settings
        let db = Database::create(&db_path)?;

        // Initialize tables
        let write_txn = db.begin_write()?;
        {
            // Create tables if they don't exist
            let _ = write_txn.open_table(CACHE_TABLE)?;
            let _ = write_txn.open_table(METADATA_TABLE)?;
        }
        write_txn.commit()?;

        Ok(Self { db })
    }

    /// Get a cached build by key.
    ///
    /// Returns `CacheMiss` if the key doesn't exist.
    pub fn get(&self, key: &CacheKey) -> Result<CachedBuild, CacheError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        let value = table.get(key.as_hex())?.ok_or(CacheError::CacheMiss)?;

        let bytes = value.value();

        // Deserialize
        let cached: CachedBuild = bincode::deserialize(bytes)
            .map_err(|e| CacheError::DeserializationError(e.to_string()))?;

        // Validate metadata
        if !cached.metadata.is_compatible() {
            return Err(CacheError::VersionMismatch {
                expected: super::serialize::CACHE_FORMAT_VERSION,
                found: cached.metadata.format_version,
            });
        }

        Ok(cached)
    }

    /// Store a cached build by key.
    pub fn put(&self, key: &CacheKey, build: &CachedBuild) -> Result<(), CacheError> {
        // Serialize
        let bytes =
            bincode::serialize(build).map_err(|e| CacheError::SerializationError(e.to_string()))?;

        // Write to database
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            table.insert(key.as_hex(), bytes.as_slice())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Remove a cached build by key.
    pub fn remove(&self, key: &CacheKey) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(CACHE_TABLE)?;
            table.remove(key.as_hex())?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Check if a key exists in the cache.
    pub fn contains(&self, key: &CacheKey) -> Result<bool, CacheError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        Ok(table.get(key.as_hex())?.is_some())
    }

    /// Clear all cached entries.
    pub fn clear(&self) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write()?;
        {
            // Drop and recreate the table to clear it
            write_txn.delete_table(CACHE_TABLE)?;
            let _ = write_txn.open_table(CACHE_TABLE)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get the number of cached entries.
    pub fn len(&self) -> Result<usize, CacheError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(CACHE_TABLE)?;

        // Count entries by iterating
        let count = table.iter()?.count();
        Ok(count)
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> Result<bool, CacheError> {
        Ok(self.len()? == 0)
    }

    /// Set a metadata value.
    pub fn set_metadata(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(METADATA_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;

        Ok(())
    }

    /// Get a metadata value.
    pub fn get_metadata(&self, key: &str) -> Result<Option<String>, CacheError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(METADATA_TABLE)?;

        Ok(table.get(key)?.map(|v| v.value().to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_store() -> (CacheStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let store = CacheStore::open(dir.path()).unwrap();
        (store, dir)
    }

    #[test]
    fn test_store_open_creates_directory() {
        let dir = TempDir::new().unwrap();
        let cache_dir = dir.path().join("new_cache");

        let _store = CacheStore::open(&cache_dir).unwrap();

        assert!(cache_dir.exists());
        assert!(cache_dir.join("cache.redb").exists());
    }

    #[test]
    fn test_cache_miss() {
        let (store, _dir) = create_test_store();

        let key = CacheKey::from_hex("nonexistent");
        let result = store.get(&key);

        assert!(matches!(result, Err(CacheError::CacheMiss)));
    }

    #[test]
    fn test_contains() {
        let (store, _dir) = create_test_store();

        let key = CacheKey::from_hex("test_key");

        assert!(!store.contains(&key).unwrap());
    }

    #[test]
    fn test_metadata() {
        let (store, _dir) = create_test_store();

        store.set_metadata("version", "1.0").unwrap();
        let value = store.get_metadata("version").unwrap();

        assert_eq!(value, Some("1.0".to_string()));
    }

    #[test]
    fn test_clear() {
        let (store, _dir) = create_test_store();

        // Initially empty
        assert!(store.is_empty().unwrap());

        // Clear should work even when empty
        store.clear().unwrap();
        assert!(store.is_empty().unwrap());
    }
}
