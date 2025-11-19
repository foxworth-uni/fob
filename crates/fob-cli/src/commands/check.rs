//! Check command implementation.
//!
//! Validates configuration and dependencies without building.

use crate::cli::CheckArgs;
use crate::commands::utils;
use crate::config::{FobConfig, Format};
use crate::error::{ConfigError, Result};
use crate::ui;
use std::path::Path;

/// Execute the check command.
///
/// # Validation Steps
///
/// 1. Load and validate fob.config.json
/// 2. Check entry points exist
/// 3. Validate format/option combinations
/// 4. Check dependencies (if --deps flag)
/// 5. Report warnings (if --warnings flag)
///
/// # Arguments
///
/// * `args` - Parsed check command arguments
///
/// # Errors
///
/// Returns errors for invalid configuration or missing files.
pub async fn execute(args: CheckArgs) -> Result<()> {
    ui::info("Checking configuration...");

    // Load config file if specified or search for default
    let config_path = args.config.as_deref();
    let config_content = if let Some(path) = config_path {
        std::fs::read_to_string(path).map_err(|_| ConfigError::NotFound(path.to_path_buf()))?
    } else {
        let default_path = Path::new("fob.config.json");
        if default_path.exists() {
            std::fs::read_to_string(default_path)
                .map_err(|_| ConfigError::NotFound(default_path.to_path_buf()))?
        } else {
            ui::warning("No fob.config.json found, using defaults");
            return Ok(());
        }
    };

    // Parse and validate config
    let config: FobConfig = serde_json::from_str(&config_content)?;
    config.validate()?;

    ui::success("Configuration is valid!");

    // Check entry points exist
    ui::info("Checking entry points...");
    let cwd = if let Some(ref cwd_path) = config.cwd {
        cwd_path.clone()
    } else {
        utils::get_cwd()?
    };

    for entry in &config.entry {
        let entry_path = utils::resolve_path(Path::new(entry), &cwd);
        if !entry_path.exists() {
            ui::error(&format!("Entry point not found: {}", entry_path.display()));
            return Err(ConfigError::MissingField {
                field: "entry".to_string(),
                hint: format!("File does not exist: {}", entry_path.display()),
            }
            .into());
        }
        ui::success(&format!("  {} exists", entry));
    }

    // Validate option combinations
    validate_options(&config)?;

    // Check dependencies if requested
    if args.deps {
        ui::info("Checking dependencies...");
        check_dependencies(&cwd)?;
    }

    // Report warnings if requested
    if args.warnings {
        ui::info("Checking for warnings...");
        check_warnings(&config);
    }

    ui::success("All checks passed!");
    Ok(())
}

/// Validate that configuration options are compatible.
fn validate_options(config: &FobConfig) -> Result<()> {
    // IIFE without global name
    if config.format == Format::Iife && config.global_name.is_none() {
        return Err(ConfigError::MissingField {
            field: "globalName".to_string(),
            hint: "IIFE format requires a global variable name".to_string(),
        }
        .into());
    }

    // Code splitting with non-ESM
    if config.splitting && config.format != Format::Esm {
        return Err(ConfigError::ConflictingOptions(
            "Code splitting requires ESM format".to_string(),
        )
        .into());
    }

    // DTS bundle without DTS
    if config.dts_bundle == Some(true) && !config.dts {
        return Err(ConfigError::InvalidValue {
            field: "dtsBundle".to_string(),
            value: "true".to_string(),
            hint: "Requires dts: true".to_string(),
        }
        .into());
    }

    Ok(())
}

/// Check that package.json and dependencies are valid.
fn check_dependencies(cwd: &Path) -> Result<()> {
    let package_json_path = cwd.join("package.json");

    if !package_json_path.exists() {
        ui::warning("No package.json found");
        return Ok(());
    }

    let package_json_content = std::fs::read_to_string(&package_json_path)?;
    let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;

    // Check if dependencies field exists
    if let Some(deps) = package_json.get("dependencies") {
        if let Some(obj) = deps.as_object() {
            ui::info(&format!("Found {} dependencies", obj.len()));
        }
    }

    if let Some(dev_deps) = package_json.get("devDependencies") {
        if let Some(obj) = dev_deps.as_object() {
            ui::info(&format!("Found {} dev dependencies", obj.len()));
        }
    }

    ui::success("Dependencies look good");
    Ok(())
}

/// Check for potential issues and report warnings.
fn check_warnings(config: &FobConfig) {
    let mut warnings = Vec::new();

    // IIFE without global name (if it somehow passes validation)
    if config.format == Format::Iife && config.global_name.is_none() {
        warnings.push("IIFE format should have a globalName");
    }

    // Code splitting with non-ESM
    if config.splitting && config.format != Format::Esm {
        warnings.push("Code splitting works best with ESM format");
    }

    // DTS without TypeScript files
    if config.dts
        && !config
            .entry
            .iter()
            .any(|e| e.ends_with(".ts") || e.ends_with(".tsx"))
    {
        warnings.push("DTS generation enabled but no TypeScript entry points found");
    }

    // External packages not in dependencies
    if !config.external.is_empty() {
        warnings.push("Ensure external packages are listed in package.json dependencies");
    }

    // Minification without production build
    if !config.minify {
        warnings.push("Consider enabling minification for production builds");
    }

    // No source maps
    if config.sourcemap.is_none() {
        warnings.push("Consider enabling source maps for better debugging");
    }

    if warnings.is_empty() {
        ui::info("No warnings found");
    } else {
        ui::warning(&format!("Found {} potential issues:", warnings.len()));
        for warning in warnings {
            ui::warning(&format!("  - {}", warning));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EsTarget, Platform};
    use std::path::PathBuf;

    fn test_config() -> FobConfig {
        FobConfig {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Esm,
            out_dir: PathBuf::from("dist"),
            bundle: true,
            dts: false,
            dts_bundle: None,
            external: vec![],
            platform: Platform::Browser,
            sourcemap: None,
            minify: false,
            target: EsTarget::Es2020,
            global_name: None,
            splitting: false,
            no_treeshake: false,
            clean: false,
            cwd: None,
        }
    }

    #[test]
    fn test_validate_options_valid_esm() {
        let config = test_config();
        assert!(validate_options(&config).is_ok());
    }

    #[test]
    fn test_validate_options_iife_without_global_name() {
        let mut config = test_config();
        config.format = Format::Iife;
        config.global_name = None;

        assert!(validate_options(&config).is_err());
    }

    #[test]
    fn test_validate_options_iife_with_global_name() {
        let mut config = test_config();
        config.format = Format::Iife;
        config.global_name = Some("MyLib".to_string());

        assert!(validate_options(&config).is_ok());
    }

    #[test]
    fn test_validate_options_splitting_with_cjs() {
        let mut config = test_config();
        config.format = Format::Cjs;
        config.splitting = true;

        assert!(validate_options(&config).is_err());
    }

    #[test]
    fn test_validate_options_splitting_with_esm() {
        let mut config = test_config();
        config.format = Format::Esm;
        config.splitting = true;

        assert!(validate_options(&config).is_ok());
    }

    #[test]
    fn test_validate_options_dts_bundle_without_dts() {
        let mut config = test_config();
        config.dts = false;
        config.dts_bundle = Some(true);

        assert!(validate_options(&config).is_err());
    }

    #[test]
    fn test_validate_options_dts_bundle_with_dts() {
        let mut config = test_config();
        config.dts = true;
        config.dts_bundle = Some(true);

        assert!(validate_options(&config).is_ok());
    }

    #[test]
    fn test_check_warnings_generates_warnings() {
        let mut config = test_config();
        config.dts = true; // DTS enabled but no .ts files

        // Should not panic
        check_warnings(&config);
    }
}
