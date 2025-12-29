//! Development UI generators for dev server HTML/JS

use crate::ProgramBuilder;
use crate::error::Result;
use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;

/// HTML builder for generating dev server HTML
pub struct HtmlBuilder {
    allocator: Allocator,
}

impl<'a> HtmlBuilder {
    /// Create a new HTML builder
    pub fn new(allocator: Allocator) -> Self {
        Self { allocator }
    }

    /// Generate index.html for dev server
    ///
    /// Creates a minimal HTML shell that loads the JavaScript bundle
    /// and includes hot reload script.
    pub fn index_html(&self, entry_point: Option<&str>) -> Result<String> {
        let script_src = entry_point.unwrap_or("/virtual_gumbo-client-entry.js");

        // Generate HTML as a string (for now, since HTML isn't JS AST)
        // TODO: Consider creating an HTML AST builder if needed
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="description" content="Fob application">
    <!-- React 19 will inject title and additional meta tags here -->
    <title>Fob Dev Server</title>
</head>
<body>
    <!-- React root mount point -->
    <div id="root"></div>

    <!-- Application bundle -->
    <script type="module" src="{}"></script>

    <!-- Hot reload for development -->
    <script src="/__fob_reload__.js"></script>
</body>
</html>"#,
            script_src
        );

        Ok(html)
    }

    /// Generate error overlay HTML
    ///
    /// Creates an HTML error page displayed in the browser when builds fail.
    /// Auto-dismisses and reloads when the next build succeeds.
    pub fn error_overlay(&self, error: &str) -> Result<String> {
        let escaped_error = html_escape(error);

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Build Error - Fob Dev Server</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            font-family: 'Menlo', 'Monaco', 'Courier New', monospace;
            background: #1a1a1a;
            color: #e8e8e8;
            padding: 20px;
            line-height: 1.6;
        }}

        .container {{
            max-width: 1200px;
            margin: 0 auto;
        }}

        .header {{
            background: #ff4444;
            color: white;
            padding: 20px 30px;
            border-radius: 8px 8px 0 0;
            font-size: 18px;
            font-weight: bold;
            display: flex;
            align-items: center;
            gap: 10px;
        }}

        .icon {{
            font-size: 24px;
        }}

        .error-content {{
            background: #2a2a2a;
            padding: 30px;
            border-radius: 0 0 8px 8px;
            border: 2px solid #ff4444;
            border-top: none;
        }}

        pre {{
            background: #1a1a1a;
            padding: 20px;
            border-radius: 4px;
            overflow-x: auto;
            white-space: pre-wrap;
            word-wrap: break-word;
            color: #ff6b6b;
            border-left: 4px solid #ff4444;
        }}

        .actions {{
            margin-top: 20px;
            display: flex;
            gap: 10px;
        }}

        button {{
            background: #4a9eff;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
            transition: background 0.2s;
        }}

        button:hover {{
            background: #3a8eef;
        }}

        button:active {{
            background: #2a7edf;
        }}

        .info {{
            margin-top: 20px;
            padding: 15px;
            background: #2a3a4a;
            border-radius: 4px;
            border-left: 4px solid #4a9eff;
            color: #a8c8e8;
        }}

        .footer {{
            margin-top: 30px;
            text-align: center;
            color: #888;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <span class="icon">⚠️</span>
            <span>Build Error</span>
        </div>
        <div class="error-content">
            <pre>{}</pre>
            <div class="actions">
                <button onclick="location.reload()">Retry Build</button>
            </div>
            <div class="info">
                This error will automatically disappear once the build succeeds.
                The page will reload automatically.
            </div>
        </div>
        <div class="footer">
            Fob Dev Server
        </div>
    </div>

    <script>
        // Connect to SSE for auto-reload on success
        const eventSource = new EventSource('/__fob_sse__');

        eventSource.addEventListener('message', (event) => {{
            try {{
                const data = JSON.parse(event.data);
                if (data.type === 'BuildCompleted') {{
                    // Build succeeded, reload the page
                    location.reload();
                }}
            }} catch (e) {{
                console.error('Failed to parse SSE event:', e);
            }}
        }});

        eventSource.addEventListener('error', () => {{
            // Reconnect on error (handled by EventSource automatically)
            console.log('SSE connection lost, will reconnect...');
        }});
    </script>
</body>
</html>"#,
            escaped_error
        );

        Ok(html)
    }

    /// Inject an import map script tag into HTML
    ///
    /// Adds a `<script type="importmap">` tag with the provided JSON content
    /// before the closing `</head>` tag, or at the beginning if no `</head>` is found.
    ///
    /// # Arguments
    ///
    /// * `html` - Existing HTML content
    /// * `import_map_json` - JSON string for the import map
    ///
    /// # Returns
    ///
    /// HTML string with import map injected
    pub fn inject_import_map(&self, html: &str, import_map_json: &str) -> String {
        let snippet = format!(r#"<script type="importmap">{}</script>"#, import_map_json);

        if let Some(idx) = html.find("</head>") {
            let (head, tail) = html.split_at(idx);
            format!("{}{}{}", head, snippet, tail)
        } else {
            format!("{}{}", snippet, html)
        }
    }

    /// Generate route manifest JavaScript
    ///
    /// Creates a JavaScript module exporting route configuration
    /// with lazy-loaded components.
    pub fn route_manifest(&self, routes: &'a [RouteSpec]) -> Result<String> {
        let mut js = ProgramBuilder::new(&self.allocator);
        let route_objects: Vec<_> = routes
            .iter()
            .map(|route| {
                js.object(vec![
                    js.prop("path", js.string(route.path.as_str())),
                    js.prop("id", js.string(route.id.as_str())),
                    js.prop(
                        "component",
                        js.call(
                            js.ident("lazy"),
                            vec![js.arg(js.arrow_fn(
                                vec![],
                                js.call(
                                    js.ident("import"),
                                    vec![js.arg(js.string(route.file.as_str()))],
                                ),
                            ))],
                        ),
                    ),
                ])
            })
            .collect();

        let routes_array = js.array(route_objects);
        let routes_decl = js.const_decl("routes", routes_array);
        let export_default = js.export_default(js.ident("routes"));

        js.push(routes_decl);
        js.push(Statement::from(export_default));

        js.generate(&Default::default())
    }
}

/// Route specification for manifest generation
#[derive(Debug, Clone)]
pub struct RouteSpec {
    /// Route path (e.g., "/", "/about", "/blog/:slug")
    pub path: String,
    /// Route ID (e.g., "index", "about", "blog_post")
    pub id: String,
    /// Component file path (e.g., "./routes/index.tsx")
    pub file: String,
}

/// HTML-escape a string to prevent XSS attacks
fn html_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#x27;".to_string(),
            _ => c.to_string(),
        })
        .collect()
}
