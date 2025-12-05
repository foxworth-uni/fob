//! # fob-target
//!
//! Deployment target adapters for Fob bundler.
//!
//! This crate provides adapters for different deployment targets (Vercel, Cloudflare, Browser)
//! that configure module resolution, export conditions, and output generation.

pub mod detection;
pub mod target;
pub mod targets;

pub use target::DeploymentTarget;
pub use targets::*;
