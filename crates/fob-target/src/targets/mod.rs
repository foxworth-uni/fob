//! Built-in deployment targets.

pub mod browser;
pub mod cloudflare;
pub mod vercel;

pub use browser::BrowserTarget;
pub use cloudflare::CloudflareWorkersTarget;
pub use vercel::VercelNodeTarget;
