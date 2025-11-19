//! # fob-store
//!
//! Fob storage backend - SurrealDB graph storage implementation.
//!
//! This crate provides persistent graph storage using SurrealDB with platform-specific
//! backends: kv-mem for WASM/browser, kv-rocksdb for native.

pub mod storage;

// Re-export storage types
pub use storage::{GraphQueries, GraphStorage, StorageError};

