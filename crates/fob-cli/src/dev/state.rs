//! Shared state for the development server.
//!
//! Provides thread-safe access to build artifacts, client connections,
//! and build status using parking_lot RwLock for better performance.

use fob_core::builders::asset_registry::AssetRegistry;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

/// Build status tracking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStatus {
    /// No build has been performed yet
    NotStarted,
    /// Build is currently in progress
    InProgress { started_at: Instant },
    /// Build completed successfully
    Success { duration_ms: u64 },
    /// Build failed with error
    Failed { error: String },
}

impl BuildStatus {
    /// Check if build is currently running.
    pub fn is_in_progress(&self) -> bool {
        matches!(self, BuildStatus::InProgress { .. })
    }

    /// Check if last build succeeded.
    pub fn is_success(&self) -> bool {
        matches!(self, BuildStatus::Success { .. })
    }

    /// Check if build has not started yet.
    pub fn is_not_started(&self) -> bool {
        matches!(self, BuildStatus::NotStarted)
    }

    /// Get error message if failed.
    pub fn error(&self) -> Option<&str> {
        match self {
            BuildStatus::Failed { error } => Some(error),
            _ => None,
        }
    }
}

/// In-memory bundle cache for serving without disk I/O.
///
/// Maps file paths to their bundled content (JavaScript, source maps, etc.).
/// This allows instant serving while background disk writes happen.
#[derive(Debug, Clone, Default)]
pub struct BundleCache {
    /// Cached file contents: path -> (content, content-type)
    files: HashMap<String, (Vec<u8>, String)>,
}

impl BundleCache {
    /// Create a new empty cache.
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Insert a file into the cache.
    ///
    /// # Arguments
    ///
    /// * `path` - URL path (e.g., "/index.js")
    /// * `content` - File content as bytes
    /// * `content_type` - MIME type (e.g., "application/javascript")
    pub fn insert(&mut self, path: String, content: Vec<u8>, content_type: String) {
        self.files.insert(path, (content, content_type));
    }

    /// Get a file from the cache.
    ///
    /// # Returns
    ///
    /// Option containing (content, content_type) if found
    pub fn get(&self, path: &str) -> Option<&(Vec<u8>, String)> {
        self.files.get(path)
    }

    /// Clear all cached files.
    pub fn clear(&mut self) {
        self.files.clear();
    }

    /// Get number of cached files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Find the first JavaScript entry point file.
    ///
    /// Returns the path of the first .js or .mjs file that is not a source map
    /// or internal file (starting with /__).
    ///
    /// # Returns
    ///
    /// Option containing the entry point path if found
    pub fn find_entry_point(&self) -> Option<String> {
        for path in self.files.keys() {
            if path.ends_with(".js") || path.ends_with(".mjs") {
                // Skip source maps and internal files
                if !path.contains(".map") && !path.starts_with("/__") {
                    return Some(path.clone());
                }
            }
        }
        None
    }
}

/// Client connection tracker for Server-Sent Events.
///
/// Tracks connected clients to broadcast reload events.
pub type ClientRegistry = Arc<RwLock<HashMap<usize, tokio::sync::mpsc::Sender<String>>>>;

/// Shared development server state.
///
/// All fields use parking_lot::RwLock for thread-safe access with minimal overhead.
/// Multiple readers can access simultaneously, writers get exclusive access.
pub struct DevServerState {
    /// Current build status
    pub status: RwLock<BuildStatus>,

    /// In-memory bundle cache
    pub cache: RwLock<BundleCache>,

    /// Connected SSE clients
    pub clients: ClientRegistry,

    /// Next client ID
    pub next_client_id: RwLock<usize>,

    /// Asset registry for serving static assets
    pub asset_registry: RwLock<Arc<AssetRegistry>>,

    /// Output directory for serving files from disk
    pub out_dir: PathBuf,
}

impl DevServerState {
    /// Create new dev server state.
    ///
    /// # Arguments
    ///
    /// * `out_dir` - Output directory for serving files from disk
    pub fn new(out_dir: PathBuf) -> Self {
        Self {
            status: RwLock::new(BuildStatus::NotStarted),
            cache: RwLock::new(BundleCache::new()),
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: RwLock::new(0),
            asset_registry: RwLock::new(Arc::new(AssetRegistry::new())),
            out_dir,
        }
    }

    /// Create new dev server state with a specific asset registry.
    #[cfg(test)]
    pub fn new_with_registry(registry: Arc<AssetRegistry>) -> Self {
        Self {
            status: RwLock::new(BuildStatus::NotStarted),
            cache: RwLock::new(BundleCache::new()),
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: RwLock::new(0),
            asset_registry: RwLock::new(registry),
            out_dir: PathBuf::from("dist"),
        }
    }

    /// Update build status to in-progress.
    pub fn start_build(&self) {
        *self.status.write() = BuildStatus::InProgress {
            started_at: Instant::now(),
        };
    }

    /// Update build status to success.
    ///
    /// # Arguments
    ///
    /// * `duration_ms` - Build duration in milliseconds
    pub fn complete_build(&self, duration_ms: u64) {
        *self.status.write() = BuildStatus::Success { duration_ms };
    }

    /// Update build status to failed.
    ///
    /// # Arguments
    ///
    /// * `error` - Error message describing the failure
    pub fn fail_build(&self, error: String) {
        *self.status.write() = BuildStatus::Failed { error };
    }

    /// Get current build status.
    pub fn get_status(&self) -> BuildStatus {
        self.status.read().clone()
    }

    /// Update the bundle cache.
    ///
    /// # Arguments
    ///
    /// * `new_cache` - New cache to replace current cache
    pub fn update_cache(&self, new_cache: BundleCache) {
        *self.cache.write() = new_cache;
    }

    /// Get a file from the cache.
    pub fn get_cached_file(&self, path: &str) -> Option<(Vec<u8>, String)> {
        self.cache.read().get(path).cloned()
    }

    /// Clear the cache.
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Register a new SSE client.
    ///
    /// # Returns
    ///
    /// Client ID and receiver for events
    pub fn register_client(&self) -> (usize, tokio::sync::mpsc::Receiver<String>) {
        let id = {
            let mut next_id = self.next_client_id.write();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let (tx, rx) = tokio::sync::mpsc::channel(100);
        self.clients.write().insert(id, tx);

        (id, rx)
    }

    /// Unregister an SSE client.
    pub fn unregister_client(&self, id: usize) {
        self.clients.write().remove(&id);
    }

    /// Broadcast an event to all connected clients.
    ///
    /// # Arguments
    ///
    /// * `event` - Event data to send (will be JSON serialized)
    pub async fn broadcast(&self, event: &crate::dev::DevEvent) {
        let json = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
        // Don't format as SSE here - let axum's Event::data() handle it

        // Get all client senders
        let clients = self.clients.read().clone();

        // Collect failed client IDs first to avoid modifying HashMap during iteration
        let mut failed_ids = Vec::new();

        // Send to each client (non-blocking)
        for (id, tx) in clients {
            if tx.send(json.clone()).await.is_err() {
                // Client disconnected, mark for removal
                failed_ids.push(id);
            }
        }

        // Remove failed clients after iteration
        for id in failed_ids {
            self.unregister_client(id);
        }
    }

    /// Get number of connected clients.
    pub fn client_count(&self) -> usize {
        self.clients.read().len()
    }

    /// Get the asset registry.
    pub fn asset_registry(&self) -> Arc<AssetRegistry> {
        Arc::clone(&*self.asset_registry.read())
    }

    /// Update the asset registry.
    pub fn update_asset_registry(&self, registry: Arc<AssetRegistry>) {
        *self.asset_registry.write() = registry;
    }

    /// Get the output directory.
    pub fn get_out_dir(&self) -> &PathBuf {
        &self.out_dir
    }
}

impl Default for DevServerState {
    fn default() -> Self {
        Self::new(PathBuf::from("dist"))
    }
}

/// Shared state handle for passing around the application.
pub type SharedState = Arc<DevServerState>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_status_is_in_progress() {
        let status = BuildStatus::InProgress {
            started_at: Instant::now(),
        };
        assert!(status.is_in_progress());
        assert!(!status.is_success());
        assert!(status.error().is_none());
    }

    #[test]
    fn test_build_status_success() {
        let status = BuildStatus::Success { duration_ms: 100 };
        assert!(!status.is_in_progress());
        assert!(status.is_success());
        assert!(status.error().is_none());
    }

    #[test]
    fn test_build_status_failed() {
        let status = BuildStatus::Failed {
            error: "Test error".to_string(),
        };
        assert!(!status.is_in_progress());
        assert!(!status.is_success());
        assert_eq!(status.error(), Some("Test error"));
    }

    #[test]
    fn test_bundle_cache_operations() {
        let mut cache = BundleCache::new();
        assert!(cache.is_empty());

        cache.insert(
            "/index.js".to_string(),
            b"console.log('test')".to_vec(),
            "application/javascript".to_string(),
        );

        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let file = cache.get("/index.js");
        assert!(file.is_some());

        let (content, content_type) = file.unwrap();
        assert_eq!(content, b"console.log('test')");
        assert_eq!(content_type, "application/javascript");

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_dev_server_state_build_lifecycle() {
        let state = DevServerState::new(PathBuf::from("dist"));

        assert!(matches!(state.get_status(), BuildStatus::NotStarted));

        state.start_build();
        assert!(state.get_status().is_in_progress());

        state.complete_build(150);
        assert!(state.get_status().is_success());

        state.fail_build("Test error".to_string());
        assert_eq!(state.get_status().error(), Some("Test error"));
    }

    #[test]
    fn test_dev_server_state_cache() {
        let state = DevServerState::new(PathBuf::from("dist"));

        let mut cache = BundleCache::new();
        cache.insert(
            "/test.js".to_string(),
            b"test".to_vec(),
            "application/javascript".to_string(),
        );

        state.update_cache(cache);

        let file = state.get_cached_file("/test.js");
        assert!(file.is_some());

        state.clear_cache();
        assert!(state.get_cached_file("/test.js").is_none());
    }

    #[tokio::test]
    async fn test_client_registration() {
        let state = Arc::new(DevServerState::new(PathBuf::from("dist")));

        let (id1, _rx1) = state.register_client();
        let (id2, _rx2) = state.register_client();

        assert_eq!(state.client_count(), 2);
        assert_ne!(id1, id2);

        state.unregister_client(id1);
        assert_eq!(state.client_count(), 1);
    }
}
