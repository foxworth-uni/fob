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

/// Maximum candidate length for security validation
const MAX_CANDIDATE_LENGTH: usize = 256;

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
    /// Detection priority:
    /// 1. `packageManager` field in package.json (Node.js/Corepack standard)
    /// 2. Lockfiles in current directory
    /// 3. Lockfiles in parent directories (for monorepos)
    /// 4. Default to npm if package.json exists
    #[allow(clippy::disallowed_methods)] // Need filesystem access for lockfile detection
    fn detect(project_root: &Path) -> Option<Self> {
        // PRIORITY 1: Check package.json's packageManager field (Corepack standard)
        // This is the most reliable indicator in modern Node.js projects
        let package_json_path = project_root.join("package.json");
        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(pm_field) = parsed.get("packageManager").and_then(|v| v.as_str()) {
                    // packageManager field format: "pnpm@8.0.0", "npm@9.0.0", etc.
                    if pm_field.starts_with("pnpm") {
                        return Some(Self::Pnpm);
                    } else if pm_field.starts_with("npm") {
                        return Some(Self::Npm);
                    } else if pm_field.starts_with("bun") {
                        return Some(Self::Bun);
                    }
                    // Note: Deno doesn't use package.json packageManager field
                }
            }
        }

        // PRIORITY 2: Check for lockfiles in current directory
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

        // PRIORITY 3: Walk up directory tree to find workspace lockfiles
        // This handles monorepo cases where lockfile is at workspace root
        let mut current = project_root.parent();
        while let Some(parent) = current {
            if parent.join("pnpm-lock.yaml").exists() && parent.join("pnpm-workspace.yaml").exists() {
                return Some(Self::Pnpm);
            }
            if parent.join("pnpm-lock.yaml").exists() {
                return Some(Self::Pnpm);
            }
            // Only check parent directories up to a reasonable depth
            if parent.join("package-lock.json").exists() {
                return Some(Self::Npm);
            }
            if parent.join("bun.lockb").exists() {
                return Some(Self::Bun);
            }
            
            // Stop at filesystem root or after checking a few levels
            current = parent.parent();
            if current.is_none() {
                break;
            }
        }

        // PRIORITY 4: Default to npm if package.json exists but no lockfile found
        if package_json_path.exists() {
            return Some(Self::Npm);
        }

        None
    }

    /// Build the command to execute Tailwind CLI via this package manager
    fn build_command(&self) -> Vec<&'static str> {
        match self {
            Self::Pnpm => vec!["pnpm", "exec", "tailwindcss"],
            Self::Npm => vec!["npx", "--no-install", "tailwindcss"],
            Self::Bun => vec!["bunx", "tailwindcss"],
            Self::Deno => vec!["deno", "run", "--allow-all", "npm:tailwindcss"],
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
        let package_manager = PackageManager::detect(&project_root)
            .ok_or_else(|| {
                GeneratorError::cli_not_found(vec![
                    project_root.join("package.json"),
                    project_root.join("pnpm-lock.yaml"),
                    project_root.join("package-lock.json"),
                ])
            })?;

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
    /// Skips package manager detection and uses the specified one.
    ///
    /// # Arguments
    ///
    /// * `package_manager` - The package manager to use
    /// * `project_root` - Root directory of the project
    pub fn with_package_manager(
        package_manager: PackageManager,
        project_root: PathBuf,
    ) -> Self {
        Self {
            package_manager,
            project_root,
            config_file: None,
            minify: false,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
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

    /// Set custom timeout in seconds
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Validate a CSS class candidate for security
    ///
    /// Rejects candidates that:
    /// - Exceed maximum length
    /// - Contain path traversal sequences
    /// - Contain shell metacharacters
    /// - Contain null bytes
    fn validate_candidate(candidate: &str) -> Result<(), GeneratorError> {
        // Check length
        if candidate.len() > MAX_CANDIDATE_LENGTH {
            return Err(GeneratorError::invalid_candidate(
                candidate,
                format!("exceeds maximum length of {}", MAX_CANDIDATE_LENGTH),
            ));
        }

        // Check for path traversal (.. sequences)
        // Note: We allow forward slashes as they can appear in arbitrary Tailwind values
        // like content-['path/to/file'] or bg-[url('/image.jpg')]
        if candidate.contains("..") {
            return Err(GeneratorError::invalid_candidate(
                candidate,
                "contains path traversal sequence",
            ));
        }

        // Check for null bytes
        if candidate.contains('\0') {
            return Err(GeneratorError::invalid_candidate(
                candidate,
                "contains null byte",
            ));
        }

        // Check for shell metacharacters
        let forbidden_chars = ['$', '`', '!', '&', '|', ';', '<', '>', '(', ')', '{', '}'];
        if candidate.chars().any(|c| forbidden_chars.contains(&c)) {
            return Err(GeneratorError::invalid_candidate(
                candidate,
                "contains shell metacharacters",
            ));
        }

        Ok(())
    }

    /// Generate CSS for the given class candidates
    ///
    /// # Arguments
    ///
    /// * `candidates` - List of CSS class candidates to generate
    ///
    /// # Returns
    ///
    /// Generated CSS as a string
    ///
    /// # Implementation
    ///
    /// This method:
    /// 1. Validates all candidates for security
    /// 2. Builds a command using the detected package manager
    /// 3. Spawns the package manager to execute Tailwind CLI
    /// 4. Writes candidates to stdin (one per line)
    /// 5. Reads generated CSS from stdout
    pub async fn generate(&self, candidates: &[String]) -> Result<String, GeneratorError> {
        // Validate all candidates first
        for candidate in candidates {
            Self::validate_candidate(candidate)?;
        }

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

        // Use stdin/stdout for communication
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set working directory to project root
        cmd.current_dir(&self.project_root);

        // Spawn the process
        let mut child = cmd
            .spawn()
            .map_err(GeneratorError::spawn_failed)?;

        // Get stdin handle
        let mut stdin = child.stdin.take().ok_or_else(|| {
            GeneratorError::spawn_failed(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "Failed to capture stdin",
            ))
        })?;

        // Write candidates to stdin (one per line)
        let input = candidates.join("\n");
        stdin
            .write_all(input.as_bytes())
            .await
            .map_err(GeneratorError::spawn_failed)?;
        drop(stdin); // Close stdin to signal EOF

        // Wait for process with timeout
        let output = timeout(Duration::from_secs(self.timeout_secs), child.wait_with_output())
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
    fn test_validate_candidate_valid() {
        assert!(TailwindGenerator::validate_candidate("text-blue-500").is_ok());
        assert!(TailwindGenerator::validate_candidate("hover:bg-red-200").is_ok());
        assert!(TailwindGenerator::validate_candidate("sm:flex").is_ok());
    }

    #[test]
    fn test_validate_candidate_too_long() {
        let long_candidate = "a".repeat(300);
        let result = TailwindGenerator::validate_candidate(&long_candidate);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GeneratorError::InvalidCandidate { .. }));
    }

    #[test]
    fn test_validate_candidate_path_traversal() {
        // Path traversal with .. should fail
        assert!(TailwindGenerator::validate_candidate("../etc/passwd").is_err());
        
        // Forward slashes are allowed (for Tailwind arbitrary values like bg-[url('/image.jpg')])
        assert!(TailwindGenerator::validate_candidate("foo/bar").is_ok());
        
        // Backslashes are also allowed (Windows paths in arbitrary values)
        assert!(TailwindGenerator::validate_candidate("foo\\bar").is_ok());
    }

    #[test]
    fn test_validate_candidate_null_byte() {
        assert!(TailwindGenerator::validate_candidate("foo\0bar").is_err());
    }

    #[test]
    fn test_validate_candidate_shell_metacharacters() {
        assert!(TailwindGenerator::validate_candidate("foo$bar").is_err());
        assert!(TailwindGenerator::validate_candidate("foo`bar").is_err());
        assert!(TailwindGenerator::validate_candidate("foo;bar").is_err());
        assert!(TailwindGenerator::validate_candidate("foo|bar").is_err());
    }
}
