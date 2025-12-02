//! Tailwind CSS CLI integration for generating CSS from class candidates

use crate::error::GeneratorError;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// Maximum allowed size for CLI output (50 MB)
const MAX_OUTPUT_SIZE: usize = 50 * 1024 * 1024;

/// Default timeout for CLI operations (30 seconds)
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Supported package managers for running Tailwind CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageManager {
    /// pnpm package manager
    Pnpm,
    /// npm package manager
    Npm,
    /// Bun package manager/runtime
    Bun,
    /// Deno runtime
    Deno,
}

impl PackageManager {
    /// Detect package manager from package.json and lockfiles
    ///
    /// Priority: packageManager field > lockfiles > default to npm
    #[allow(clippy::disallowed_methods)]
    fn detect(project_root: &Path) -> Option<Self> {
        let package_json_path = project_root.join("package.json");

        // Check packageManager field in package.json (Corepack standard)
        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(pm) = parsed.get("packageManager").and_then(|v| v.as_str()) {
                    if pm.starts_with("pnpm") {
                        return Some(Self::Pnpm);
                    } else if pm.starts_with("bun") {
                        return Some(Self::Bun);
                    } else if pm.starts_with("npm") {
                        return Some(Self::Npm);
                    }
                }
            }
        }

        // Check lockfiles
        if project_root.join("pnpm-lock.yaml").exists() {
            return Some(Self::Pnpm);
        }
        if project_root.join("bun.lockb").exists() {
            return Some(Self::Bun);
        }
        if project_root.join("deno.lock").exists() {
            return Some(Self::Deno);
        }
        if project_root.join("package-lock.json").exists() {
            return Some(Self::Npm);
        }

        // Default to npm if package.json exists
        if package_json_path.exists() {
            return Some(Self::Npm);
        }

        None
    }

    /// Build the command to execute Tailwind CLI via this package manager
    /// Note: Tailwind v4 CLI is in @tailwindcss/cli package, but the binary is named "tailwindcss"
    fn build_command(&self) -> Vec<&'static str> {
        match self {
            Self::Pnpm => vec!["pnpm", "exec", "tailwindcss"],
            Self::Npm => vec!["npx", "--no-install", "tailwindcss"],
            Self::Bun => vec!["bunx", "tailwindcss"],
            Self::Deno => vec!["deno", "run", "--allow-all", "npm:@tailwindcss/cli"],
        }
    }

    /// Get the name of this package manager for display
    pub fn name(&self) -> &'static str {
        match self {
            Self::Pnpm => "pnpm",
            Self::Npm => "npm",
            Self::Bun => "bun",
            Self::Deno => "deno",
        }
    }

    /// Check if this package manager binary is available on the system
    pub async fn validate_binary(&self) -> Result<(), GeneratorError> {
        let binary_name = match self {
            Self::Pnpm => "pnpm",
            Self::Npm => "npx",
            Self::Bun => "bunx",
            Self::Deno => "deno",
        };

        #[cfg(unix)]
        let check_cmd = "which";
        #[cfg(windows)]
        let check_cmd = "where";

        let output = Command::new(check_cmd)
            .arg(binary_name)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(GeneratorError::spawn_failed)?;

        if !output.success() {
            return Err(GeneratorError::PackageManagerNotFound {
                package_manager: self.name().to_string(),
                binary_name: binary_name.to_string(),
            });
        }

        Ok(())
    }
}

/// Tailwind CSS CLI generator
///
/// Manages detection and execution of the Tailwind CSS CLI to generate
/// CSS from class candidates. Uses package managers (pnpm/npm/bun/deno) to
/// run the CLI from project dependencies via stdin/stdout communication.
#[derive(Debug)]
pub struct TailwindGenerator {
    /// Package manager to use for running Tailwind CLI
    package_manager: PackageManager,

    /// Root directory of the project (for resolving config files)
    project_root: PathBuf,

    /// Optional path to Tailwind config file
    config_file: Option<PathBuf>,

    /// Whether to minify output CSS
    minify: bool,

    /// Timeout for CLI operations in seconds
    timeout_secs: u64,
}

impl TailwindGenerator {
    /// Create a new TailwindGenerator by auto-detecting the package manager
    ///
    /// # Arguments
    ///
    /// * `project_root` - Root directory of the project
    ///
    /// # Returns
    ///
    /// A new generator instance, or an error if no package manager is detected
    pub async fn new(project_root: PathBuf) -> Result<Self, GeneratorError> {
        let package_manager = PackageManager::detect(&project_root).ok_or_else(|| {
            GeneratorError::cli_not_found(vec![
                project_root.join("package.json"),
                project_root.join("pnpm-lock.yaml"),
                project_root.join("package-lock.json"),
            ])
        })?;

        // Validate binary exists before proceeding
        package_manager.validate_binary().await?;

        Ok(Self {
            package_manager,
            project_root,
            config_file: None,
            minify: false,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        })
    }

    /// Create a generator with an explicit package manager
    ///
    /// Validates that the package manager binary exists before creating.
    pub async fn with_package_manager(
        package_manager: PackageManager,
        project_root: PathBuf,
    ) -> Result<Self, GeneratorError> {
        package_manager.validate_binary().await?;

        Ok(Self {
            package_manager,
            project_root,
            config_file: None,
            minify: false,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        })
    }

    /// Set the Tailwind config file path
    pub fn with_config(mut self, config_file: PathBuf) -> Self {
        self.config_file = Some(config_file);
        self
    }

    /// Enable minification
    pub fn with_minify(mut self, minify: bool) -> Self {
        self.minify = minify;
        self
    }

    /// Generate CSS from the given input CSS content
    ///
    /// # Arguments
    ///
    /// * `input_css` - CSS content to process, containing `@tailwind` directives
    ///
    /// # Returns
    ///
    /// Generated CSS as a string
    ///
    /// # Implementation
    ///
    /// This method:
    /// 1. Builds a command using the detected package manager
    /// 2. Spawns the package manager to execute Tailwind CLI
    /// 3. Writes the input CSS to stdin
    /// 4. Reads generated CSS from stdout
    pub async fn generate_from_input(&self, input_css: &str) -> Result<String, GeneratorError> {
        // Build command using package manager
        let cmd_parts = self.package_manager.build_command();
        let mut cmd = Command::new(cmd_parts[0]);

        // Add remaining command parts (exec, tailwindcss, etc.)
        for part in &cmd_parts[1..] {
            cmd.arg(part);
        }

        // Add config file if specified
        if let Some(config) = &self.config_file {
            cmd.arg("--config").arg(config);
        }

        // Add minify flag if enabled
        if self.minify {
            cmd.arg("--minify");
        }

        // v4 CLI: -i - reads from stdin, -o - writes to stdout
        cmd.arg("-i").arg("-").arg("-o").arg("-");

        // Use stdin/stdout for communication
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory to project root
        cmd.current_dir(&self.project_root);

        // Spawn the process
        let mut child = cmd.spawn().map_err(GeneratorError::spawn_failed)?;

        // Get stdin handle
        let mut stdin = child.stdin.take().ok_or_else(|| {
            GeneratorError::spawn_failed(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to capture stdin",
            ))
        })?;

        // Write input CSS to stdin
        stdin
            .write_all(input_css.as_bytes())
            .await
            .map_err(GeneratorError::spawn_failed)?;
        drop(stdin); // Close stdin to signal EOF

        // Wait for process with timeout
        let output = timeout(
            Duration::from_secs(self.timeout_secs),
            child.wait_with_output(),
        )
        .await
        .map_err(|_| GeneratorError::timeout(self.timeout_secs))?
        .map_err(GeneratorError::spawn_failed)?;

        // Check exit status
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code().unwrap_or(-1);
            return Err(GeneratorError::cli_exit_error(exit_code, stderr));
        }

        // Check output size
        if output.stdout.len() > MAX_OUTPUT_SIZE {
            return Err(GeneratorError::output_too_large(
                output.stdout.len(),
                MAX_OUTPUT_SIZE,
            ));
        }

        // Parse output as UTF-8
        String::from_utf8(output.stdout).map_err(GeneratorError::parse_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_pnpm() {
        let cmd = PackageManager::Pnpm.build_command();
        assert_eq!(cmd, vec!["pnpm", "exec", "tailwindcss"]);
    }

    #[test]
    fn test_build_command_npm() {
        let cmd = PackageManager::Npm.build_command();
        assert_eq!(cmd, vec!["npx", "--no-install", "tailwindcss"]);
    }

    #[test]
    fn test_build_command_bun() {
        let cmd = PackageManager::Bun.build_command();
        assert_eq!(cmd, vec!["bunx", "tailwindcss"]);
    }

    #[test]
    fn test_build_command_deno() {
        let cmd = PackageManager::Deno.build_command();
        assert_eq!(
            cmd,
            vec!["deno", "run", "--allow-all", "npm:@tailwindcss/cli"]
        );
    }

    #[test]
    fn test_package_manager_name() {
        assert_eq!(PackageManager::Pnpm.name(), "pnpm");
        assert_eq!(PackageManager::Npm.name(), "npm");
        assert_eq!(PackageManager::Bun.name(), "bun");
        assert_eq!(PackageManager::Deno.name(), "deno");
    }

    #[test]
    fn test_package_manager_detect_from_pnpm_lockfile() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_pnpm");
        let _ = std::fs::create_dir_all(&temp_dir);
        let lockfile = temp_dir.join("pnpm-lock.yaml");
        std::fs::write(&lockfile, "lockfileVersion: 9.0").unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Pnpm));

        // Cleanup
        let _ = std::fs::remove_file(&lockfile);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_from_npm_lockfile() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_npm");
        let _ = std::fs::create_dir_all(&temp_dir);
        let lockfile = temp_dir.join("package-lock.json");
        std::fs::write(&lockfile, "{}").unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Npm));

        // Cleanup
        let _ = std::fs::remove_file(&lockfile);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_from_bun_lockfile() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_bun");
        let _ = std::fs::create_dir_all(&temp_dir);
        let lockfile = temp_dir.join("bun.lockb");
        std::fs::write(&lockfile, "").unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Bun));

        // Cleanup
        let _ = std::fs::remove_file(&lockfile);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_from_deno_lockfile() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_deno");
        let _ = std::fs::create_dir_all(&temp_dir);
        let lockfile = temp_dir.join("deno.lock");
        std::fs::write(&lockfile, "{}").unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Deno));

        // Cleanup
        let _ = std::fs::remove_file(&lockfile);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_from_package_json_field() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_corepack");
        let _ = std::fs::create_dir_all(&temp_dir);
        let pkg_json = temp_dir.join("package.json");
        std::fs::write(&pkg_json, r#"{"packageManager": "pnpm@9.0.0"}"#).unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Pnpm));

        // Cleanup
        let _ = std::fs::remove_file(&pkg_json);
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_none_empty_dir() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_empty");
        let _ = std::fs::create_dir_all(&temp_dir);

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, None);

        // Cleanup
        let _ = std::fs::remove_dir(&temp_dir);
    }

    #[test]
    fn test_package_manager_detect_defaults_to_npm_with_package_json() {
        let temp_dir = std::env::temp_dir().join("tailwind_test_default_npm");
        let _ = std::fs::create_dir_all(&temp_dir);
        let pkg_json = temp_dir.join("package.json");
        std::fs::write(&pkg_json, r#"{"name": "test"}"#).unwrap();

        let result = PackageManager::detect(&temp_dir);
        assert_eq!(result, Some(PackageManager::Npm));

        // Cleanup
        let _ = std::fs::remove_file(&pkg_json);
        let _ = std::fs::remove_dir(&temp_dir);
    }
}
