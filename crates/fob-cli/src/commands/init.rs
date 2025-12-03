//! Init command implementation.
//!
//! Creates new Fob projects from templates.

use crate::cli::InitArgs;
use crate::commands::{templates, utils};
use crate::error::{CliError, Result};
use crate::ui;
use std::fs;
use std::path::Path;

/// Execute the init command.
///
/// # Process
///
/// 1. Determine project name
/// 2. Select template (interactive or from args)
/// 3. Detect package manager
/// 4. Create project directory structure
/// 5. Generate files from template
/// 6. Show next steps
///
/// # Arguments
///
/// * `args` - Parsed init command arguments
///
/// # Errors
///
/// Returns errors for:
/// - Invalid project names
/// - Directory already exists
/// - File write failures
pub async fn execute(args: InitArgs) -> Result<()> {
    // Step 1: Determine project name
    let project_name = determine_project_name(&args)?;
    validate_project_name(&project_name)?;

    ui::info(&format!("Creating project: {}", project_name));

    // Step 2: Select template
    let template = select_template(&args)?;
    ui::info(&format!("Using template: {}", template.name()));

    // Step 3: Create project directory
    let project_dir = Path::new(&project_name);
    if project_dir.exists() {
        return Err(CliError::InvalidArgument(format!(
            "Directory '{}' already exists",
            project_name
        )));
    }

    fs::create_dir(project_dir)?;
    ui::success(&format!("Created directory: {}", project_name));

    // Step 4: Generate project files
    generate_project_files(project_dir, &project_name, template)?;

    // Step 5: Detect package manager
    let pkg_mgr = if args.use_pnpm {
        utils::PackageManager::Pnpm
    } else if args.use_yarn {
        utils::PackageManager::Yarn
    } else {
        // Default to npm or auto-detect
        utils::PackageManager::Npm
    };

    // Step 6: Display next steps
    print_next_steps(&project_name, pkg_mgr);

    ui::success("Project created successfully!");
    Ok(())
}

/// Determine the project name from args or current directory.
fn determine_project_name(args: &InitArgs) -> Result<String> {
    if let Some(ref name) = args.name {
        Ok(name.clone())
    } else {
        // Use current directory name
        let cwd = utils::get_cwd()?;
        let dir_name = cwd
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CliError::InvalidArgument("Invalid directory name".to_string()))?;
        Ok(dir_name.to_string())
    }
}

/// Reserved package names that cannot be used (npm reserved names).
const RESERVED_NAMES: &[&str] = &[
    "node_modules",
    "favicon.ico",
    "index.js",
    "index.html",
    "package.json",
    "tsconfig.json",
    "fob.config.json",
    ".git",
    ".gitignore",
    ".DS_Store",
];

/// Validate project name follows npm package naming rules.
fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(CliError::InvalidArgument(
            "Project name cannot be empty".to_string(),
        ));
    }

    // Check for reserved names
    if RESERVED_NAMES.contains(&name) {
        return Err(CliError::InvalidArgument(format!(
            "Project name '{}' is reserved and cannot be used",
            name
        )));
    }

    // Check for invalid characters
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(CliError::InvalidArgument(
            "Project name can only contain letters, numbers, hyphens, and underscores".to_string(),
        ));
    }

    // Cannot start with a dot or number
    // Note: empty check above guarantees name is non-empty
    if name.starts_with('.') || name.as_bytes()[0].is_ascii_digit() {
        return Err(CliError::InvalidArgument(
            "Project name cannot start with a dot or number".to_string(),
        ));
    }

    Ok(())
}

/// Select template based on args or interactive prompt.
///
/// Validates that the user-provided template name is valid and returns the
/// corresponding Template enum variant. Security: Input validation prevents
/// arbitrary file system operations.
fn select_template(args: &InitArgs) -> Result<templates::Template> {
    if let Some(ref template_name) = args.template {
        templates::Template::from_str(template_name).ok_or_else(|| {
            CliError::InvalidArgument(format!(
                "Invalid template '{}'. Available: library, app, component-library, meta-framework",
                template_name
            ))
        })
    } else if args.yes {
        // Default to library when using --yes
        Ok(templates::Template::Library)
    } else {
        // In a real implementation, this would show an interactive prompt
        // For now, default to library
        ui::info("Defaulting to 'library' template (use --template to specify)");
        Ok(templates::Template::Library)
    }
}

/// Generate all project files from template.
fn generate_project_files(
    project_dir: &Path,
    project_name: &str,
    template: templates::Template,
) -> Result<()> {
    ui::info("Generating files...");

    // Create src directory
    let src_dir = project_dir.join("src");
    fs::create_dir(&src_dir)?;

    // Generate package.json
    let package_json = templates::package_json(project_name, template);
    fs::write(project_dir.join("package.json"), package_json)?;
    ui::success("  Created package.json");

    // Generate tsconfig.json
    let tsconfig = templates::tsconfig_json(template);
    fs::write(project_dir.join("tsconfig.json"), tsconfig)?;
    ui::success("  Created tsconfig.json");

    // Generate fob.config.json
    let joy_config = templates::joy_config_json(template);
    fs::write(project_dir.join("fob.config.json"), joy_config)?;
    ui::success("  Created fob.config.json");

    // Generate source file
    let source_file = templates::source_file(template);
    let source_filename = match template {
        templates::Template::Library => "index.ts",
        templates::Template::App => "main.ts",
        templates::Template::ComponentLibrary => "index.ts",
        templates::Template::MetaFramework => "index.ts",
    };
    fs::write(src_dir.join(source_filename), source_file)?;
    ui::success(&format!("  Created src/{}", source_filename));

    // Generate template-specific files
    match template {
        templates::Template::App => {
            // Create index.html
            let index_html = templates::index_html(project_name);
            fs::write(project_dir.join("index.html"), index_html)?;
            ui::success("  Created index.html");

            // Create app.css
            let app_css = templates::app_css();
            fs::write(src_dir.join("app.css"), app_css)?;
            ui::success("  Created src/app.css");
        }
        templates::Template::Library => {
            // No additional files for library
        }
        templates::Template::ComponentLibrary => {
            // Create Button.tsx
            let button_content = templates::button_component();
            fs::write(src_dir.join("Button.tsx"), button_content)?;
            ui::success("  Created src/Button.tsx");
        }
        templates::Template::MetaFramework => {
            // Create router.ts and server.ts
            let router_content = templates::router_module();
            let server_content = templates::server_module();
            fs::write(src_dir.join("router.ts"), router_content)?;
            ui::success("  Created src/router.ts");
            fs::write(src_dir.join("server.ts"), server_content)?;
            ui::success("  Created src/server.ts");
        }
    }

    // Generate .gitignore
    let gitignore = templates::gitignore();
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    ui::success("  Created .gitignore");

    // Generate README.md
    let readme = templates::readme(project_name, template);
    fs::write(project_dir.join("README.md"), readme)?;
    ui::success("  Created README.md");

    Ok(())
}

/// Print next steps for the user.
fn print_next_steps(project_name: &str, pkg_mgr: utils::PackageManager) {
    eprintln!();
    ui::info("Next steps:");
    eprintln!();
    eprintln!("  cd {}", project_name);
    eprintln!("  {}", pkg_mgr.install_cmd());
    eprintln!("  {} run dev", pkg_mgr.command());
    eprintln!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name_valid() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("project123").is_ok());
    }

    #[test]
    fn test_validate_project_name_empty() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn test_validate_project_name_starts_with_dot() {
        assert!(validate_project_name(".hidden").is_err());
    }

    #[test]
    fn test_validate_project_name_starts_with_number() {
        assert!(validate_project_name("123project").is_err());
    }

    #[test]
    fn test_validate_project_name_invalid_chars() {
        assert!(validate_project_name("my@project").is_err());
        assert!(validate_project_name("my project").is_err());
        assert!(validate_project_name("my/project").is_err());
    }

    #[test]
    fn test_validate_project_name_reserved() {
        assert!(validate_project_name("node_modules").is_err());
        assert!(validate_project_name("favicon.ico").is_err());
        assert!(validate_project_name("index.js").is_err());
        assert!(validate_project_name("package.json").is_err());
    }

    #[test]
    fn test_select_template_from_args() {
        let args = InitArgs {
            name: None,
            template: Some("library".to_string()),
            yes: false,
            use_npm: false,
            use_yarn: false,
            use_pnpm: false,
        };

        assert_eq!(
            select_template(&args).unwrap(),
            templates::Template::Library
        );
    }

    #[test]
    fn test_select_template_with_yes_flag() {
        let args = InitArgs {
            name: None,
            template: None,
            yes: true,
            use_npm: false,
            use_yarn: false,
            use_pnpm: false,
        };

        assert_eq!(
            select_template(&args).unwrap(),
            templates::Template::Library
        );
    }

    #[test]
    fn test_select_template_invalid() {
        let args = InitArgs {
            name: None,
            template: Some("invalid".to_string()),
            yes: false,
            use_npm: false,
            use_yarn: false,
            use_pnpm: false,
        };

        assert!(select_template(&args).is_err());
    }
}
