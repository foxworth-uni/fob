use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::{routing::get, Json, Router};
use rust_embed::RustEmbed;
use serde_json::json;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::analysis::AnalysisDocument;

#[derive(RustEmbed)]
#[folder = "assets/plot"]
struct PlotAssets;

#[derive(Clone)]
struct ServerState {
    analysis: Arc<AnalysisDocument>,
    analysis_pretty: Arc<String>,
}

pub async fn serve(analysis: AnalysisDocument, port: u16) -> Result<()> {
    let pretty = analysis
        .to_pretty_json()
        .unwrap_or_else(|_| String::from("{}"));

    let state = ServerState {
        analysis: Arc::new(analysis),
        analysis_pretty: Arc::new(pretty),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/analysis", get(api_analysis))
        .fallback_service(ServeDir::new("."))
        .with_state(state);

    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    let listener = TcpListener::bind(addr).await?;

    println!(
        "📊 Joy dashboard available at http://{}:{}",
        addr.ip(),
        addr.port()
    );

    axum::serve(listener, app).await?;
    Ok(())
}

async fn index(State(state): State<ServerState>) -> impl IntoResponse {
    match PlotAssets::get("standalone.html") {
        Some(asset) => {
            let html = String::from_utf8_lossy(asset.data.as_ref()).replace(
                "const ANALYSIS_DATA = null;",
                &format!("const ANALYSIS_DATA = {};", state.analysis_pretty),
            );
            Html(html).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Missing embedded visualization template",
        )
            .into_response(),
    }
}

async fn api_analysis(State(state): State<ServerState>) -> impl IntoResponse {
    Json(json!({
        "analysis": state.analysis.as_ref(),
    }))
}
