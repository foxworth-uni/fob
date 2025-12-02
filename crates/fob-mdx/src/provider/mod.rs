//! Data provider infrastructure for MDX props
//!
//! This module defines the core abstractions for data providers
//! that resolve prop definitions from MDX frontmatter.
//!
//! # Overview
//!
//! The provider system enables MDX pages to fetch external data:
//!
//! ```yaml
//! ---
//! props:
//!   stars: github.repo("owner/name").stargazers_count @refresh=60s
//! ---
//! ```
//!
//! Frameworks implement the [`Provider`] trait to resolve these expressions.
//!
//! # Architecture
//!
//! - **[`Provider`]**: Trait for data providers (e.g., GitHub, Notion)
//! - **[`ProviderRegistry`]**: Manages provider instances
//! - **[`ProviderError`]**: Error types for provider operations

mod error;
mod registry;
mod traits;

pub use error::ProviderError;
pub use registry::ProviderRegistry;
pub use traits::Provider;
