use anyhow::Result;
use askama::Template;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod mdx;

use mdx::MdxCompiler;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BlogPost {
    slug: String,
    title: String,
    description: Option<String>,
    author: Option<String>,
    date: Option<DateTime<Utc>>,
    tags: Vec<String>,
    content: String,
    html: String,
}

#[derive(Clone)]
struct AppState {
    posts: Arc<RwLock<HashMap<String, BlogPost>>>,
    compiler: Arc<MdxCompiler>,
}

impl AppState {
    async fn new() -> Result<Self> {
        let compiler = Arc::new(MdxCompiler::new());
        let posts = Arc::new(RwLock::new(HashMap::new()));

        let state = Self { posts, compiler };
        state.load_posts().await?;

        Ok(state)
    }

    async fn load_posts(&self) -> Result<()> {
        let posts_dir = PathBuf::from("examples/blog-axum/posts");

        if !posts_dir.exists() {
            warn!("Posts directory does not exist: {:?}", posts_dir);
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&posts_dir).await?;
        let mut posts_map = HashMap::new();

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) != Some("mdx") {
                continue;
            }

            match self.load_post(&path).await {
                Ok(post) => {
                    info!("Loaded post: {}", post.slug);
                    posts_map.insert(post.slug.clone(), post);
                }
                Err(e) => {
                    warn!("Failed to load post {:?}: {}", path, e);
                }
            }
        }

        let mut posts = self.posts.write().await;
        *posts = posts_map;

        Ok(())
    }

    async fn load_post(&self, path: &PathBuf) -> Result<BlogPost> {
        let content = tokio::fs::read_to_string(path).await?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled")
            .to_string();

        let result = self.compiler.compile(&content)?;

        let post = BlogPost {
            slug: slug.clone(),
            title: result
                .frontmatter
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or(&slug)
                .to_string(),
            description: result
                .frontmatter
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            author: result
                .frontmatter
                .get("author")
                .and_then(|v| v.as_str())
                .map(String::from),
            date: result
                .frontmatter
                .get("date")
                .and_then(|v| v.as_str())
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            tags: result
                .frontmatter
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            content,
            html: result.code,
        };

        Ok(post)
    }

    async fn get_all_posts(&self) -> Vec<BlogPost> {
        let posts = self.posts.read().await;
        let mut posts_vec: Vec<_> = posts.values().cloned().collect();

        // Sort by date, newest first
        posts_vec.sort_by(|a, b| b.date.cmp(&a.date));

        posts_vec
    }

    async fn get_post(&self, slug: &str) -> Option<BlogPost> {
        let posts = self.posts.read().await;
        posts.get(slug).cloned()
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    posts: Vec<BlogPost>,
}

#[derive(Template)]
#[template(path = "post.html")]
struct PostTemplate {
    post: BlogPost,
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate;

struct HtmlTemplate<T>(T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template: {}", err),
            )
                .into_response(),
        }
    }
}

async fn index(axum::extract::State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    let posts = state.get_all_posts().await;
    HtmlTemplate(IndexTemplate { posts })
}

async fn post(
    Path(slug): Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    match state.get_post(&slug).await {
        Some(post) => HtmlTemplate(PostTemplate { post }).into_response(),
        None => (StatusCode::NOT_FOUND, HtmlTemplate(NotFoundTemplate)).into_response(),
    }
}

async fn reload_posts(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    match state.load_posts().await {
        Ok(_) => (StatusCode::OK, "Posts reloaded successfully").into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to reload posts: {}", e),
        )
            .into_response(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "blog_axum=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize app state
    let state = AppState::new().await?;

    info!("Loaded {} blog posts", state.posts.read().await.len());

    // Build router
    let app = Router::new()
        .route("/", get(index))
        .route("/post/{slug}", get(post))
        .route("/api/reload", get(reload_posts))
        .nest_service("/static", ServeDir::new("examples/blog-axum/static"))
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8888));
    info!("Starting server on http://{}", addr);
    info!("Visit http://localhost:8888 to view the blog");
    info!("POST to http://localhost:8888/api/reload to reload posts");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
