use chrono::Utc;
use serde::Serialize;

use crate::{
    error::{DocsError, Result},
    model::Documentation,
};

/// Pretty-printed JSON representation of the documentation model including metadata.
pub fn render_json(doc: &Documentation) -> Result<String> {
    let payload = JsonPayload {
        version: env!("CARGO_PKG_VERSION"),
        generated_at: Utc::now().to_rfc3339(),
        documentation: doc,
    };

    serde_json::to_string_pretty(&payload).map_err(|error| DocsError::Other {
        message: error.to_string(),
    })
}

#[derive(Serialize)]
struct JsonPayload<'a> {
    version: &'static str,
    generated_at: String,
    documentation: &'a Documentation,
}
