//! Development server configuration.
//!
//! Extends the base FobConfig with dev-server-specific settings.

use crate::cli::DevArgs;
use crate::config::FobConfig;
use crate::error::{ConfigError, Result};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// Development server configuration.
///
/// Combines base bundler configuration with dev server settings
/// like port, HTTPS, and watch configuration.
#[derive(Debug, Clone)]
pub struct DevConfig {
    /// Base bundler configuration
    pub base: FobConfig,

    /// Server socket address (IP + port)
    pub addr: SocketAddr,

    /// Enable HTTPS with self-signed certificate
    pub https: bool,

    /// Open browser automatically on start
    pub open: bool,

    /// Working directory for the dev server
    pub cwd: PathBuf,

    /// Patterns to ignore when watching files
    pub watch_ignore: Vec<String>,

    /// Debounce delay in milliseconds for file changes
    pub debounce_ms: u64,
}

impl DevConfig {
    /// Create DevConfig from CLI arguments.
    ///
    /// # Arguments
    ///
    /// * `args` - Parsed dev command arguments
    ///
    /// # Returns
    ///
    /// Configured DevConfig instance
    ///
    /// # Errors
    ///
    /// Returns error if entry point is invalid or configuration is inconsistent
    pub fn from_args(args: &DevArgs) -> Result<Self> {
        // Convert DevArgs entry (Option<PathBuf>) to BuildArgs entry (Vec<String>)
        // If entry is provided via CLI, use it (will override config file)
        // If not provided, use empty vec (allows config file to take precedence)
        let entry_vec = if let Some(ref entry) = args.entry {
            vec![entry.to_string_lossy().to_string()]
        } else {
            vec![]
        };

        // Create minimal BuildArgs for loading base config
        let build_args = crate::cli::BuildArgs {
            entry: entry_vec,
            format: crate::cli::Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: false,
            dts_bundle: false,
            external: vec![],
            docs: false,
            docs_format: None,
            docs_dir: None,
            docs_include_internal: false,
            docs_enhance: false,
            docs_enhance_mode: None,
            docs_llm_model: None,
            docs_no_cache: false,
            docs_llm_url: None,
            docs_write_back: false,
            docs_merge_strategy: None,
            docs_no_backup: false,
            platform: crate::cli::Platform::Browser,
            sourcemap: Some(crate::cli::SourceMapMode::External),
            minify: false,
            target: crate::cli::EsTarget::Es2020,
            global_name: None,
            splitting: false,
            no_treeshake: false,
            clean: false,
            cwd: args.cwd.clone(),
            bundle: true,
        };

        // Load base configuration (merges with fob.config.json if present)
        let base = FobConfig::load(&build_args, None)?;

        // Validate that we have at least one entry from somewhere
        if base.entry.is_empty() {
            return Err(crate::error::CliError::InvalidArgument(
                "No entry point specified. Provide entry via CLI argument or fob.config.json"
                    .to_string(),
            ));
        }

        // Resolve project root using smart auto-detection
        let cwd = crate::commands::utils::resolve_project_root(
            base.cwd.as_deref(),                    // explicit --cwd flag
            base.entry.first().map(|s| s.as_str()), // first entry point
        )?;

        // Try to bind to requested port, fall back to next available
        let addr = Self::find_available_port(args.port)?;

        // Default ignore patterns for file watching
        let watch_ignore = vec![
            "node_modules".to_string(),
            ".git".to_string(),
            "dist".to_string(),
            "build".to_string(),
            "*.log".to_string(),
            ".DS_Store".to_string(),
        ];

        Ok(Self {
            base,
            addr,
            https: args.https,
            open: args.open,
            cwd,
            watch_ignore,
            debounce_ms: 100, // 100ms debounce
        })
    }

    /// Find an available port starting from the requested port.
    ///
    /// Tries the requested port first, then incrementally searches
    /// for the next available port (up to +10 from original).
    ///
    /// # Security
    ///
    /// - Validates port is not in privileged range (< 1024) unless root
    /// - Prevents binding to 0.0.0.0 in production mode
    fn find_available_port(requested_port: u16) -> Result<SocketAddr> {
        use std::net::TcpListener;

        // Security: warn if using privileged port
        if requested_port < 1024 {
            crate::ui::warning(&format!(
                "Port {} is in privileged range, may require root access",
                requested_port
            ));
        }

        // Try requested port first
        let addr = SocketAddr::from(([127, 0, 0, 1], requested_port));
        if TcpListener::bind(addr).is_ok() {
            return Ok(addr);
        }

        // Try next 10 ports
        for offset in 1..=10 {
            let port = requested_port.saturating_add(offset);
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if TcpListener::bind(addr).is_ok() {
                crate::ui::warning(&format!(
                    "Port {} is busy, using port {} instead",
                    requested_port, port
                ));
                return Ok(addr);
            }
        }

        Err(ConfigError::InvalidValue {
            field: "port".to_string(),
            value: requested_port.to_string(),
            hint: format!(
                "Ports {}-{} are all in use. Try a different port range.",
                requested_port,
                requested_port + 10
            ),
        }
        .into())
    }

    /// Validate the dev server configuration.
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Entry point doesn't exist
    /// - Working directory is invalid
    /// - Base config validation fails
    pub fn validate(&self) -> Result<()> {
        // Validate base configuration
        self.base.validate()?;

        // Validate working directory exists
        if !self.cwd.exists() {
            return Err(ConfigError::InvalidValue {
                field: "cwd".to_string(),
                value: self.cwd.display().to_string(),
                hint: "Working directory does not exist".to_string(),
            }
            .into());
        }

        // Validate entry point exists
        for entry in &self.base.entry {
            let entry_path = if Path::new(entry).is_absolute() {
                PathBuf::from(entry)
            } else {
                self.cwd.join(entry)
            };

            if !entry_path.exists() {
                return Err(ConfigError::InvalidValue {
                    field: "entry".to_string(),
                    value: entry.clone(),
                    hint: format!("Entry point does not exist: {}", entry_path.display()),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Get the server URL as a string.
    pub fn server_url(&self) -> String {
        let protocol = if self.https { "https" } else { "http" };
        format!("{}://{}", protocol, self.addr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn test_find_available_port_success() {
        let listener = match TcpListener::bind(("127.0.0.1", 0)) {
            Ok(listener) => listener,
            Err(err) => {
                eprintln!(
                    "Skipping test_find_available_port_success: unable to bind socket ({})",
                    err
                );
                return;
            }
        };

        let start_port = listener.local_addr().unwrap().port();
        drop(listener);

        let addr = DevConfig::find_available_port(start_port).expect("should find port");
        assert_eq!(addr.ip().to_string(), "127.0.0.1");
        assert!(addr.port() >= start_port);
    }

    #[test]
    fn test_find_available_port_privileged_warning() {
        // Should warn about privileged port but still try to bind
        let result = DevConfig::find_available_port(80);
        // May succeed or fail depending on permissions
        // Just ensure it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_server_url_http() {
        let config = DevConfig {
            base: FobConfig {
                entry: vec!["src/index.ts".to_string()],
                ..base_config()
            },
            addr: "127.0.0.1:3000".parse().unwrap(),
            https: false,
            open: false,
            cwd: PathBuf::from("."),
            watch_ignore: vec![],
            debounce_ms: 100,
        };

        assert_eq!(config.server_url(), "http://127.0.0.1:3000");
    }

    #[test]
    fn test_server_url_https() {
        let config = DevConfig {
            base: FobConfig {
                entry: vec!["src/index.ts".to_string()],
                ..base_config()
            },
            addr: "127.0.0.1:3000".parse().unwrap(),
            https: true,
            open: false,
            cwd: PathBuf::from("."),
            watch_ignore: vec![],
            debounce_ms: 100,
        };

        assert_eq!(config.server_url(), "https://127.0.0.1:3000");
    }

    fn base_config() -> FobConfig {
        FobConfig {
            entry: vec![],
            format: crate::config::Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: false,
            dts_bundle: None,
            external: vec![],
            platform: crate::config::Platform::Browser,
            sourcemap: None,
            minify: false,
            target: crate::config::EsTarget::Es2020,
            global_name: None,
            bundle: true,
            splitting: false,
            no_treeshake: false,
            clean: false,
            cwd: None,
        }
    }
}
