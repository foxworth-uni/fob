//! Development server with hot reload via Server-Sent Events.
//!
//! Serves bundled files from memory cache and provides SSE endpoint
//! for push-based reload notifications.

use crate::dev::{error_overlay, DevConfig, SharedState};
use crate::error::Result;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Response, Sse},
    routing::get,
    Router,
};
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

/// Development server.
pub struct DevServer {
    /// Server configuration
    config: DevConfig,
    /// Shared application state
    state: SharedState,
}

impl DevServer {
    /// Create a new development server.
    ///
    /// # Arguments
    ///
    /// * `config` - Server configuration
    /// * `state` - Shared state for caching and client tracking
    pub fn new(config: DevConfig, state: SharedState) -> Self {
        Self { config, state }
    }

    /// Start the development server.
    ///
    /// Creates an axum router with:
    /// - SSE endpoint for reload events
    /// - Static file serving from cache
    /// - HTML injection for reload script
    /// - CORS headers (allow all origins for dev)
    ///
    /// # Returns
    ///
    /// Server handle that can be gracefully shut down
    ///
    /// # Errors
    ///
    /// Returns error if server cannot bind to configured address
    pub async fn start(self) -> Result<()> {
        let addr = self.config.addr;
        let server_url = self.config.server_url();

        // Build router
        let app = self.build_router();

        // Create listener
        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            crate::error::CliError::Server(format!("Failed to bind to {}: {}", addr, e))
        })?;

        crate::ui::success(&format!("Development server running at {}", server_url));

        // Start server
        axum::serve(listener, app)
            .await
            .map_err(|e| crate::error::CliError::Server(format!("Server error: {}", e)))?;

        Ok(())
    }

    /// Build the axum router with all routes.
    fn build_router(self) -> Router {
        let state = self.state.clone();

        Router::new()
            // SSE endpoint for reload events
            .route("/__fob_sse__", get(handle_sse))
            // Reload client script
            .route("/__fob_reload__.js", get(handle_reload_script))
            // Asset serving (WASM, images, etc.)
            .route("/__fob_assets__/{*path}", get(crate::dev::handle_asset))
            // Favicon handler to prevent 404s
            .route("/favicon.ico", get(handle_favicon))
            // All other routes serve bundled files
            .fallback(handle_request)
            .layer(
                // CORS: Allow all origins for dev (standard practice)
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state)
    }
}

/// Handle SSE connections for reload events.
async fn handle_sse(
    State(state): State<SharedState>,
) -> Sse<
    impl tokio_stream::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
> {
    use axum::response::sse::Event;

    // Register this client
    let (id, rx) = state.register_client();

    crate::ui::info(&format!("Client {} connected via SSE", id));

    // Notify about connection
    let _ = state
        .broadcast(&crate::dev::DevEvent::ClientConnected { id })
        .await;

    // Convert receiver to stream for SSE
    let stream = ReceiverStream::new(rx).map(|data| Ok(Event::default().data(data)));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

/// Serve the reload client script.
async fn handle_reload_script() -> impl IntoResponse {
    const RELOAD_SCRIPT: &str = include_str!("../../assets/dev/reload-client.js");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/javascript")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(RELOAD_SCRIPT))
        .unwrap()
}

/// Handle favicon requests with 204 No Content.
async fn handle_favicon() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

/// Handle all other requests (serve bundled files or error overlay).
async fn handle_request(
    State(state): State<SharedState>,
    uri: Uri,
) -> Result<impl IntoResponse, Response> {
    let path = uri.path();

    // Check build status
    let status = state.get_status();

    // If build failed, show error overlay
    if let Some(error) = status.error() {
        let html = error_overlay::generate_error_overlay(error).map_err(|e| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(format!(
                    "Failed to generate error overlay: {}",
                    e
                )))
                .unwrap()
        })?;

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(html))
            .unwrap());
    }

    // Try to serve from cache
    if let Some((content, content_type)) = state.get_cached_file(path) {
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, content_type)
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(content))
            .unwrap());
    }

    // Try to serve from disk (for assets in subdirectories)
    let file_path = state.get_out_dir().join(path.trim_start_matches('/'));
    if file_path.exists() && file_path.is_file() {
        match tokio::fs::read(&file_path).await {
            Ok(content) => {
                let content_type = determine_content_type(path);
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, content_type)
                    .header(header::CACHE_CONTROL, "no-cache")
                    .body(Body::from(content))
                    .unwrap());
            }
            Err(e) => {
                crate::ui::warning(&format!(
                    "Failed to read file {}: {}",
                    file_path.display(),
                    e
                ));
            }
        }
    }

    // Special handling for root path
    if path == "/" {
        // Try index.html or index.js
        if let Some((content, content_type)) = state.get_cached_file("/index.html") {
            let html = inject_reload_script(&content, &content_type);

            return Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .header(header::CACHE_CONTROL, "no-cache")
                .body(Body::from(html))
                .unwrap());
        }

        // Fallback: serve minimal HTML that loads the bundle
        // Find the first JavaScript file in the cache as the entry point
        let entry_point = find_entry_point_from_cache(&state);
        let html = generate_index_html(entry_point.as_deref()).map_err(|e| {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header(header::CONTENT_TYPE, "text/plain")
                .body(Body::from(format!("Failed to generate HTML: {}", e)))
                .unwrap()
        })?;

        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .header(header::CACHE_CONTROL, "no-cache")
            .body(Body::from(html))
            .unwrap());
    }

    // File not found
    Err(Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from(format!("File not found: {}", path)))
        .unwrap())
}

/// Inject reload script into HTML content.
///
/// Adds the reload client script before the closing </body> tag.
fn inject_reload_script(content: &[u8], content_type: &str) -> Vec<u8> {
    // Only inject into HTML files
    if !content_type.starts_with("text/html") {
        return content.to_vec();
    }

    let html = String::from_utf8_lossy(content);
    let script_tag = r#"<script src="/__fob_reload__.js"></script>"#;

    // Try to inject before </body>
    if let Some(pos) = html.rfind("</body>") {
        let mut result = String::with_capacity(html.len() + script_tag.len() + 10);
        result.push_str(&html[..pos]);
        result.push_str("\n  ");
        result.push_str(script_tag);
        result.push('\n');
        result.push_str(&html[pos..]);
        return result.into_bytes();
    }

    // Fallback: append at end
    let mut result = html.to_string();
    result.push('\n');
    result.push_str(script_tag);
    result.into_bytes()
}

/// Find the entry point JavaScript file from the cache.
///
/// Returns the first JavaScript file found in the cache, or None if no JS files exist.
fn find_entry_point_from_cache(state: &SharedState) -> Option<String> {
    state.cache.read().find_entry_point()
}

/// Generate a minimal index.html that loads the bundle.
///
/// This HTML template serves as the shell for the React SPA. It provides:
/// - The <div id="root"></div> where React will mount
/// - A script tag that loads the JavaScript bundle
/// - Hot reload script for development
///
/// React 19 components can render <title> and <meta> tags which will be
/// automatically hoisted into this <head> section.
///
/// # Arguments
///
/// * `entry_point` - Optional entry point script path (e.g., "/index.js")
///   If None, falls back to "/virtual_gumbo-client-entry.js"
///
/// # Errors
///
/// Returns an error if HTML generation fails. This should be treated as a bug.
fn generate_index_html(entry_point: Option<&str>) -> Result<String, String> {
    use fob_gen::{Allocator, HtmlBuilder};

    let allocator = Allocator::default();
    let html_builder = HtmlBuilder::new(&allocator);

    html_builder
        .index_html(entry_point)
        .map_err(|e| format!("Failed to generate index.html: {}", e))
}

/// Determine content type from file extension.
fn determine_content_type(path: &str) -> &'static str {
    let extension = std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "wasm" => "application/wasm",
        "js" | "mjs" => "application/javascript",
        "json" => "application/json",
        "map" => "application/json",
        "html" => "text/html; charset=utf-8",
        "css" => "text/css",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_reload_script_with_body() {
        let html = b"<html><body><h1>Test</h1></body></html>";
        let result = inject_reload_script(html, "text/html");

        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains(r#"<script src="/__fob_reload__.js"></script>"#));
        assert!(result_str.contains("</body>"));

        // Script should be before </body>
        let script_pos = result_str
            .find(r#"<script src="/__fob_reload__.js"></script>"#)
            .unwrap();
        let body_pos = result_str.find("</body>").unwrap();
        assert!(script_pos < body_pos);
    }

    #[test]
    fn test_inject_reload_script_without_body() {
        let html = b"<html><h1>Test</h1></html>";
        let result = inject_reload_script(html, "text/html");

        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains(r#"<script src="/__fob_reload__.js"></script>"#));
    }

    #[test]
    fn test_inject_reload_script_non_html() {
        let js = b"console.log('test');";
        let result = inject_reload_script(js, "application/javascript");

        // Should not modify non-HTML content
        assert_eq!(result, js);
    }

    #[test]
    fn test_generate_index_html_structure() {
        let html = generate_index_html(Some("/index.js")).expect("HTML generation should succeed");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<div id=\"root\"></div>"));
        assert!(html.contains(r#"<script type="module" src="/index.js"></script>"#));
        assert!(html.contains(r#"<script src="/__fob_reload__.js"></script>"#));
    }

    #[test]
    fn test_generate_index_html_default_entry() {
        let html = generate_index_html(None).expect("HTML generation should succeed");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html
            .contains(r#"<script type="module" src="/virtual_gumbo-client-entry.js"></script>"#));
    }
}
