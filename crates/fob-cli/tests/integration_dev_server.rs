//! Integration tests for the development server.
//!
//! Tests verify SSE connections, file serving, and rebuild functionality.

use fob_cli::cli::DevArgs;
use fob_cli::dev::{DevConfig, DevServerState};
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_dev_server_state_initialization() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();
    let out_dir = project_dir.join("dist");

    let state = DevServerState::new(out_dir.clone());

    assert_eq!(state.client_count(), 0);
    assert!(state.get_status().is_not_started());
    assert!(state.get_cached_file("/nonexistent.js").is_none());
}

#[tokio::test]
async fn test_dev_server_client_registration() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();
    let out_dir = project_dir.join("dist");

    let state = Arc::new(DevServerState::new(out_dir));

    let (id1, _rx1) = state.register_client();
    let (id2, _rx2) = state.register_client();

    assert_eq!(state.client_count(), 2);
    assert_ne!(id1, id2);

    state.unregister_client(id1);
    assert_eq!(state.client_count(), 1);
}

#[tokio::test]
async fn test_dev_server_broadcast_to_clients() {
    use fob_cli::dev::DevEvent;
    use std::sync::Arc;

    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();
    let out_dir = project_dir.join("dist");

    let state = Arc::new(DevServerState::new(out_dir));

    // Register two clients
    let (_id1, mut rx1) = state.register_client();
    let (_id2, mut rx2) = state.register_client();

    // Broadcast an event
    let event = DevEvent::BuildStarted;
    state.broadcast(&event).await;

    // Both clients should receive the event
    tokio::select! {
        msg1 = rx1.recv() => {
            assert!(msg1.is_some());
            let json = msg1.unwrap();
            assert!(json.contains("BuildStarted"));
        }
        _ = sleep(Duration::from_millis(100)) => {
            panic!("Client 1 did not receive broadcast");
        }
    }

    tokio::select! {
        msg2 = rx2.recv() => {
            assert!(msg2.is_some());
            let json = msg2.unwrap();
            assert!(json.contains("BuildStarted"));
        }
        _ = sleep(Duration::from_millis(100)) => {
            panic!("Client 2 did not receive broadcast");
        }
    }
}

#[tokio::test]
async fn test_dev_server_bundle_cache_operations() {
    use fob_cli::dev::BundleCache;

    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();
    let out_dir = project_dir.join("dist");

    let state = Arc::new(DevServerState::new(out_dir));

    let mut cache = BundleCache::new();
    cache.insert(
        "/index.js".to_string(),
        b"console.log('test')".to_vec(),
        "application/javascript".to_string(),
    );

    state.update_cache(cache);

    let file = state.get_cached_file("/index.js");
    assert!(file.is_some());

    let (content, content_type) = file.unwrap();
    assert_eq!(content, b"console.log('test')");
    assert_eq!(content_type, "application/javascript");

    // Test entry point finding
    let entry = state.cache.read().find_entry_point();
    assert_eq!(entry, Some("/index.js".to_string()));
}

#[tokio::test]
async fn test_dev_config_from_args() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create source file
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("index.ts"),
        r#"export const hello = () => console.log("Hello");"#,
    )
    .unwrap();

    let requested_port = match pick_available_port() {
        Some(port) => port,
        None => {
            eprintln!("Skipping test_dev_config_from_args: unable to reserve an available port");
            return;
        }
    };
    let args = DevArgs {
        entry: Some(PathBuf::from("src/index.ts")),
        port: requested_port,
        https: false,
        open: false,
        cwd: Some(project_dir.to_path_buf()),
    };

    let config = DevConfig::from_args(&args).unwrap();

    assert_eq!(config.base.entry, vec!["src/index.ts"]);
    assert_eq!(config.addr.port(), requested_port);
    assert_eq!(config.https, false);
    assert_eq!(config.open, false);
}

#[tokio::test]
async fn test_dev_config_loads_from_file() {
    let temp = TempDir::new().unwrap();
    let project_dir = temp.path();

    // Create source file
    let src_dir = project_dir.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(
        src_dir.join("main.ts"),
        r#"export const hello = () => console.log("Hello");"#,
    )
    .unwrap();

    // Create config file
    fs::write(
        project_dir.join("fob.config.json"),
        r#"{
            "entry": ["src/main.ts"],
            "outDir": "dist"
        }"#,
    )
    .unwrap();

    let requested_port = match pick_available_port() {
        Some(port) => port,
        None => {
            eprintln!(
                "Skipping test_dev_config_loads_from_file: unable to reserve an available port"
            );
            return;
        }
    };
    let args = DevArgs {
        entry: None, // Should load from config file
        port: requested_port,
        https: false,
        open: false,
        cwd: Some(project_dir.to_path_buf()),
    };

    let config = DevConfig::from_args(&args).unwrap();

    assert_eq!(config.base.entry, vec!["src/main.ts"]);
    assert_eq!(config.addr.port(), requested_port);
}

fn pick_available_port() -> Option<u16> {
    TcpListener::bind(("127.0.0.1", 0))
        .ok()
        .and_then(|listener| listener.local_addr().ok().map(|addr| addr.port()))
}
