/**
 * Cloudflare Worker Example with Fob Bundler (Rust)
 *
 * IMPORTANT LIMITATION:
 * This example demonstrates the Fob Rust API structure, but actual bundling
 * cannot run in Cloudflare Workers due to these constraints:
 *
 * 1. Rolldown (Fob's bundler) doesn't compile to WASM
 * 2. Cloudflare Workers blocks dynamic WASM compilation
 * 3. Edge environments have limited filesystem access
 *
 * This example shows:
 * - How to structure a Rust worker using the `worker` crate
 * - Type-safe request/response handling
 * - What fob-core API usage would look like (if it worked at edge)
 *
 * For actual edge bundling, consider:
 * - Pre-bundling at build time
 * - Using a separate bundling service
 * - Serverless functions with full Node.js runtime (not edge)
 */

use worker::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct BundleRequest {
    files: HashMap<String, String>,
    #[serde(default)]
    entries: Vec<String>,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Serialize)]
struct BundleResponse {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<BundleResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    meta: BundleMeta,
}

#[derive(Serialize)]
struct BundleResult {
    assets_count: usize,
    total_size: usize,
    modules_analyzed: usize,
}

#[derive(Serialize)]
struct BundleMeta {
    duration_ms: f64,
    timestamp: u64,
    worker: &'static str,
}

/// Example source files to bundle
fn example_files() -> HashMap<String, String> {
    let mut files = HashMap::new();
    
    files.insert(
        "index.js".to_string(),
        r#"
import { greet } from './utils.js';
import { version } from './constants.js';

console.log(greet('Cloudflare Worker (Rust)'));
console.log('Version:', version);

export default function handler() {
  return {
    message: greet('Edge'),
    version,
    timestamp: Date.now()
  };
}
"#.to_string(),
    );
    
    files.insert(
        "utils.js".to_string(),
        r#"
export function greet(name) {
  return `Hello, ${name}! Bundled with Fob (Rust) at the edge.`;
}

export function formatDate(date) {
  return new Date(date).toISOString();
}
"#.to_string(),
    );
    
    files.insert(
        "constants.js".to_string(),
        r#"
export const version = '1.0.0';
export const environment = 'cloudflare-worker-rust';
export const features = ['rust', 'edge-bundling', 'type-safe'];
"#.to_string(),
    );
    
    files
}

/// Simulate bundling for demonstration
///
/// NOTE: This is a simulation. Actual bundling with fob-core is not possible
/// in Cloudflare Workers because:
/// - Rolldown doesn't compile to WASM
/// - Cloudflare blocks dynamic WASM compilation
/// - Edge has limited filesystem access
///
/// In a real application, bundling would happen:
/// - At build time (before deployment)
/// - In a separate service with full Node.js
/// - In serverless functions (not edge)
async fn bundle_files(
    files: HashMap<String, String>,
    _entries: Vec<String>,
    format: Option<String>,
) -> Result<BundleResult> {
    // Determine bundle format
    let _bundle_format = match format.as_deref() {
        Some("cjs") => "CommonJS",
        Some("iife") => "IIFE",
        _ => "ESM",
    };

    // NOTE: If this were possible, the code would look like:
    //
    // use fob_bundler::{BuildOptions, OutputFormat};
    //
    // let mut options = BuildOptions::new_multiple(entries);
    // options = options.format(match format.as_deref() {
    //     Some("cjs") => OutputFormat::Cjs,
    //     Some("iife") => OutputFormat::Iife,
    //     _ => OutputFormat::Esm,
    // });
    //
    // // Would need to provide virtual files somehow
    // for (path, content) in files.iter() {
    //     options = options.virtual_file(path, content);
    // }
    //
    // let result = options.build().await?;
    //
    // However, this doesn't work in Cloudflare Workers.

    // Return simulated result
    Ok(BundleResult {
        assets_count: 1,
        total_size: files.values().map(|s| s.len()).sum(),
        modules_analyzed: files.len(),
    })
}

/// HTML template for the demo page
/// 
/// Demonstrates building type-safe HTML using structured formatting
/// rather than raw template strings.
fn render_html(result: &BundleResult, duration: f64) -> String {
    // Build HTML structure programmatically for type safety and maintainability
    let assets_str = result.assets_count.to_string();
    let time_str = format!("{:.0}ms", duration);
    let modules_str = result.modules_analyzed.to_string();
    let stats_cards = vec![
        ("Bundle Status", "✅ Success"),
        ("Assets Generated", &assets_str),
        ("Bundle Time", &time_str),
        ("Modules Analyzed", &modules_str),
    ];
    
    let mut html = String::new();
    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Fob Cloudflare Worker (Rust) Demo</title>
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
      background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
      min-height: 100vh;
      padding: 2rem;
    }}
    .container {{
      max-width: 1200px;
      margin: 0 auto;
      background: white;
      border-radius: 12px;
      box-shadow: 0 20px 60px rgba(0,0,0,0.3);
      overflow: hidden;
    }}
    header {{
      background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
      color: white;
      padding: 2rem;
      text-align: center;
    }}
    h1 {{ font-size: 2.5rem; margin-bottom: 0.5rem; }}
    .subtitle {{ font-size: 1.2rem; opacity: 0.9; }}
    .content {{ padding: 2rem; }}
    .stats {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
    }}
    .stat-card {{
      background: #f7fafc;
      padding: 1.5rem;
      border-radius: 8px;
      border-left: 4px solid #f093fb;
    }}
    .stat-label {{ color: #718096; font-size: 0.875rem; margin-bottom: 0.5rem; }}
    .stat-value {{ font-size: 1.5rem; font-weight: bold; color: #2d3748; }}
    .badge {{
      display: inline-block;
      padding: 0.25rem 0.75rem;
      background: #48bb78;
      color: white;
      border-radius: 12px;
      font-size: 0.75rem;
      font-weight: 600;
      margin-right: 0.5rem;
    }}
    .rust-badge {{
      background: linear-gradient(135deg, #ce422b 0%, #8b0000 100%);
    }}
    pre {{
      background: #2d3748;
      color: #e2e8f0;
      padding: 1rem;
      border-radius: 6px;
      overflow-x: auto;
      font-size: 0.875rem;
      line-height: 1.5;
    }}
    .section {{ margin-bottom: 2rem; }}
    .section h2 {{ color: #2d3748; margin-bottom: 1rem; font-size: 1.5rem; }}
  </style>
</head>
<body>
  <div class="container">
    <header>
      <h1>🦀 Fob Cloudflare Worker (Rust)</h1>
      <p class="subtitle">Edge bundling with Rust-powered Fob</p>
    </header>
    
    <div class="content">
      <div class="stats">"#);
    
    // Build stat cards programmatically
    for (label, value) in stats_cards {
        html.push_str(&format!(
            r#"
        <div class="stat-card">
          <div class="stat-label">{}</div>
          <div class="stat-value">{}</div>
        </div>"#,
            label, value
        ));
    }
    
    html.push_str(r#"
      </div>

      <div class="section">
        <h2>✨ Features</h2>
        <p>
          <span class="badge rust-badge">Rust</span>
          <span class="badge">Type Safe</span>
          <span class="badge">Edge Computing</span>
          <span class="badge">Zero Config</span>
          <span class="badge">Fast</span>
        </p>
      </div>

      <div class="section">
        <h2>⚠️ Important Note</h2>
        <div style="background: #fef3c7; padding: 1rem; border-radius: 6px; border-left: 4px solid #f59e0b; margin-bottom: 1rem;">
          <p style="margin: 0; color: #92400e; line-height: 1.6;">
            This is a <strong>demonstration</strong> of Rust worker structure. Actual bundling with Fob cannot run
            in Cloudflare Workers because Rolldown doesn't compile to WASM and Cloudflare blocks dynamic WASM compilation.
            For production, pre-bundle at build time or use a separate service.
          </p>
        </div>
      </div>

      <div class="section">
        <h2>🦀 Why Rust?</h2>
        <ul style="line-height: 1.8; color: #4a5568; margin-left: 1.5rem;">
          <li><strong>Type Safety</strong> - Compile-time guarantees</li>
          <li><strong>Performance</strong> - Native speed at the edge</li>
          <li><strong>Memory Safety</strong> - No runtime errors</li>
          <li><strong>Small Binary</strong> - Optimized WASM output</li>
        </ul>
      </div>

      <div class="section">
        <h2>📊 Bundle Statistics</h2>
        <pre>Assets Count:     {}
Total Size:       {} bytes
Modules Analyzed: {}
Duration:         {:.2}ms</pre>
      </div>

      <div class="section" style="margin-top: 2rem; padding-top: 2rem; border-top: 1px solid #e2e8f0;">
        <p style="text-align: center; color: #718096;">
          Built with <strong>Fob</strong> + <strong>Rust</strong> - Type-safe bundling at the edge
        </p>
      </div>
    </div>
  </div>
</body>
</html>"#);
    
    html
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    
    router
        // Route: GET / - Demo page
        .get_async("/", |_req, _ctx| async move {
            let start = Date::now().as_millis();
            
            let files = example_files();
            let entries = vec!["index.js".to_string()];
            
            let result = bundle_files(files, entries, None)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            
            let duration = (Date::now().as_millis() - start) as f64;
            
            Response::from_html(render_html(&result, duration))
        })
        
        // Route: GET /api/bundle - Get bundle result as JSON
        .get_async("/api/bundle", |_req, _ctx| async move {
            let start = Date::now().as_millis();
            
            let files = example_files();
            let entries = vec!["index.js".to_string()];
            
            let result = bundle_files(files, entries, None)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            
            let duration = (Date::now().as_millis() - start) as f64;
            
            let response = BundleResponse {
                success: true,
                result: Some(result),
                error: None,
                meta: BundleMeta {
                    duration_ms: duration,
                    timestamp: Date::now().as_millis(),
                    worker: "cloudflare-rust",
                },
            };
            
            Response::from_json(&response)
        })
        
        // Route: POST /api/bundle - Bundle custom code
        .post_async("/api/bundle", |mut req, _ctx| async move {
            let start = Date::now().as_millis();
            
            let body: BundleRequest = req.json().await?;
            
            let entries = if body.entries.is_empty() {
                vec!["index.js".to_string()]
            } else {
                body.entries
            };
            
            let result = bundle_files(body.files, entries, body.format)
                .await
                .map_err(|e| Error::RustError(e.to_string()))?;
            
            let duration = (Date::now().as_millis() - start) as f64;
            
            let response = BundleResponse {
                success: true,
                result: Some(result),
                error: None,
                meta: BundleMeta {
                    duration_ms: duration,
                    timestamp: Date::now().as_millis(),
                    worker: "cloudflare-rust",
                },
            };
            
            Response::from_json(&response)
        })
        
        .run(req, env)
        .await
}

