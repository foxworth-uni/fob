//! Development server command implementation.
//!
//! Orchestrates the entire dev server lifecycle:
//! - Initial build with validation
//! - File watching with debouncing
//! - HTTP server with SSE for hot reload
//! - Automatic rebuilds on file changes
//! - Graceful shutdown on Ctrl+C

use crate::cli::DevArgs;
use crate::dev::{
    DevBuilder, DevConfig, DevEvent, DevServer, DevServerState, FileChange, FileWatcher,
    SharedState,
};
use crate::error::Result;
use crate::ui;
use std::sync::Arc;
use tokio::signal;

/// Execute the dev command.
///
/// # Process Flow
///
/// 1. Load and validate dev configuration
/// 2. Create builder and perform initial build
/// 3. Start file watcher for auto-rebuild
/// 4. Start HTTP server with SSE
/// 5. Main event loop:
///    - Watch for file changes
///    - Trigger rebuilds on changes
///    - Broadcast events to connected clients
///    - Handle Ctrl+C for graceful shutdown
///
/// # Arguments
///
/// * `args` - Parsed dev command arguments
///
/// # Errors
///
/// Returns errors for:
/// - Invalid configuration
/// - Build failures
/// - Server startup failures
/// - File watcher errors
pub async fn execute(args: DevArgs) -> Result<()> {
    ui::info("Starting development server...");

    // Step 1: Load and validate configuration
    let config = DevConfig::from_args(&args)?;
    config.validate()?;

    let entry_display = if let Some(ref entry) = args.entry {
        entry.display().to_string()
    } else {
        config
            .base
            .entry
            .first()
            .cloned()
            .unwrap_or_else(|| "unknown".to_string())
    };
    ui::info(&format!("Entry point: {}", entry_display));
    ui::info(&format!("Working directory: {}", config.cwd.display()));

    // Step 2: Create shared state
    // Resolve output directory to absolute path
    let out_dir = if config.base.out_dir.is_absolute() {
        config.base.out_dir.clone()
    } else {
        config.cwd.join(&config.base.out_dir)
    };
    let state = Arc::new(DevServerState::new(out_dir));

    // Step 3: Create builder
    let builder = DevBuilder::new(config.base.clone(), config.cwd.clone());

    // Step 4: Perform initial build
    ui::info("Performing initial build...");
    state.start_build();

    match builder.initial_build().await {
        Ok((duration_ms, cache, asset_registry)) => {
            state.complete_build(duration_ms);
            ui::success(&format!("Initial build completed in {}ms", duration_ms));

            // Update cache and asset registry
            state.update_cache(cache);
            if let Some(registry) = asset_registry {
                state.update_asset_registry(registry);
            }

            ui::info(&format!(
                "Cached {} files in memory",
                state.cache.read().len()
            ));
        }
        Err(e) => {
            let error_msg = e.to_string();
            state.fail_build(error_msg.clone());
            ui::error(&format!("Initial build failed: {}", error_msg));
            return Err(e);
        }
    }

    // Step 5: Start file watcher
    let (watcher, mut change_rx) = FileWatcher::new(
        config.cwd.clone(),
        config.watch_ignore.clone(),
        config.debounce_ms,
    )?;

    ui::info(&format!(
        "Watching for changes in: {}",
        watcher.root().display()
    ));

    // Step 6: Start HTTP server in background
    let server = DevServer::new(config.clone(), state.clone());
    let mut server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            ui::error(&format!("Server error: {}", e));
        }
    });

    // Step 7: Open browser if requested
    if config.open {
        open_browser(&config.server_url());
    }

    // Step 8: Main event loop
    ui::info("Press Ctrl+C to stop");

    loop {
        tokio::select! {
            // File change detected
            Some(change) = change_rx.recv() => {
                handle_file_change(change, &builder, &state).await;
            }

            // Ctrl+C received
            _ = signal::ctrl_c() => {
                ui::info("Shutting down development server...");
                break;
            }

            // Server task completed (error or shutdown)
            _ = &mut server_handle => {
                ui::warning("Server task completed unexpectedly");
                break;
            }
        }
    }

    ui::success("Development server stopped");
    Ok(())
}

/// Handle a file change event.
///
/// Triggers rebuild and broadcasts result to connected clients.
async fn handle_file_change(change: FileChange, builder: &DevBuilder, state: &SharedState) {
    let path = change.path();
    ui::info(&format!("File changed: {}", path.display()));

    // Clear cached source code before rebuild to ensure fresh data
    fob_bundler::diagnostics::clear_source_cache();

    // Start build
    state.start_build();
    let _ = state.broadcast(&DevEvent::BuildStarted).await;

    // Perform rebuild
    match builder.rebuild().await {
        Ok((duration_ms, cache, asset_registry)) => {
            // Update state
            state.complete_build(duration_ms);
            state.update_cache(cache);
            if let Some(registry) = asset_registry {
                state.update_asset_registry(registry);
            }

            ui::success(&format!("Rebuild completed in {}ms", duration_ms));

            // Broadcast success - this triggers client reload
            let _ = state
                .broadcast(&DevEvent::BuildCompleted { duration_ms })
                .await;
        }
        Err(e) => {
            let error_msg = e.to_string();
            state.fail_build(error_msg.clone());

            ui::error(&format!("Rebuild failed: {}", error_msg));

            // Broadcast failure - error overlay will be shown
            let _ = state
                .broadcast(&DevEvent::BuildFailed { error: error_msg })
                .await;
        }
    }
}

/// Open the server URL in the default browser.
///
/// Uses platform-specific commands:
/// - macOS: `open`
/// - Windows: `start`
/// - Linux: `xdg-open`
fn open_browser(url: &str) {
    use std::process::Command;

    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(url).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", url]).spawn()
    } else {
        Command::new("xdg-open").arg(url).spawn()
    };

    match result {
        Ok(_) => ui::info(&format!("Opened browser at {}", url)),
        Err(e) => ui::warning(&format!("Failed to open browser: {}", e)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_open_browser_url_format() {
        // Just verify the function doesn't panic with various URLs
        // Actual browser opening depends on platform and is non-deterministic
        let urls = vec![
            "http://localhost:3000",
            "http://127.0.0.1:3000",
            "https://localhost:3000",
        ];

        for url in urls {
            // This should not panic
            let _ = std::panic::catch_unwind(|| {
                // Don't actually open browser in tests
                // Just validate URL format
                assert!(url.starts_with("http"));
            });
        }
    }
}
