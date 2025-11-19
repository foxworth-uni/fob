#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::validation::parse_global;
    use crate::cli::{Cli, Command};
    use clap::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_parse_global_valid_identifiers() {
        // Valid starting characters
        assert_eq!(parse_global("MyLibrary"), Ok("MyLibrary".to_string()));
        assert_eq!(parse_global("_private"), Ok("_private".to_string()));
        assert_eq!(parse_global("$jquery"), Ok("$jquery".to_string()));

        // With numbers (but not starting)
        assert_eq!(parse_global("lib123"), Ok("lib123".to_string()));
        assert_eq!(parse_global("test_123"), Ok("test_123".to_string()));

        // Mixed valid characters
        assert_eq!(parse_global("My_Lib$123"), Ok("My_Lib$123".to_string()));
    }

    #[test]
    fn test_parse_global_invalid_start() {
        // Starting with number
        assert!(parse_global("123lib").is_err());
        assert!(parse_global("1_test").is_err());

        // Starting with invalid character
        assert!(parse_global("-lib").is_err());
        assert!(parse_global(".lib").is_err());
        assert!(parse_global("@lib").is_err());
    }

    #[test]
    fn test_parse_global_invalid_characters() {
        // Hyphens
        assert!(parse_global("my-lib").is_err());

        // Dots
        assert!(parse_global("my.lib").is_err());

        // Spaces
        assert!(parse_global("my lib").is_err());

        // Special characters
        assert!(parse_global("my@lib").is_err());
        assert!(parse_global("my#lib").is_err());
        assert!(parse_global("my+lib").is_err());
    }

    #[test]
    fn test_parse_global_empty() {
        assert!(parse_global("").is_err());
        let err = parse_global("").unwrap_err();
        assert_eq!(err, "Global name cannot be empty");
    }

    #[test]
    fn test_parse_global_unicode() {
        // Unicode letters should work
        assert_eq!(parse_global("café"), Ok("café".to_string()));
        assert_eq!(parse_global("日本"), Ok("日本".to_string()));
    }

    #[test]
    fn test_format_enum_values() {
        // Verify enum values match expected strings
        use crate::cli::enums::Format;
        use clap::ValueEnum;

        let formats: Vec<_> = Format::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(formats, vec!["esm", "cjs", "iife"]);
    }

    #[test]
    fn test_platform_enum_values() {
        use crate::cli::enums::Platform;
        use clap::ValueEnum;

        let platforms: Vec<_> = Platform::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(platforms, vec!["browser", "node"]);
    }

    #[test]
    fn test_sourcemap_enum_values() {
        use crate::cli::enums::SourceMapMode;
        use clap::ValueEnum;

        let modes: Vec<_> = SourceMapMode::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(modes, vec!["inline", "external", "hidden"]);
    }

    #[test]
    fn test_estarget_enum_values() {
        use crate::cli::enums::EsTarget;
        use clap::ValueEnum;

        let targets: Vec<_> = EsTarget::value_variants()
            .iter()
            .map(|v| v.to_possible_value().unwrap().get_name().to_string())
            .collect();
        assert_eq!(
            targets,
            vec![
                "es2015", "es2016", "es2017", "es2018", "es2019", "es2020", "es2021", "es2022",
                "esnext"
            ]
        );
    }

    #[test]
    fn test_cli_verbose_quiet_conflict() {
        let result = Cli::try_parse_from(&["joy", "--verbose", "--quiet", "build", "src/index.ts"]);

        // Should fail because verbose and quiet conflict
        assert!(result.is_err());
    }

    #[test]
    fn test_build_args_defaults() {
        use crate::cli::enums::{EsTarget, Format, Platform};
        use clap::Parser;

        let args = Cli::try_parse_from(&["joy", "build", "src/index.ts"]).unwrap();

        if let Command::Build(build) = args.command {
            assert_eq!(build.entry, vec!["src/index.ts"]);
            assert_eq!(build.format, Format::Esm);
            assert_eq!(build.out_dir, PathBuf::from("dist"));
            assert_eq!(build.platform, Platform::Browser);
            assert_eq!(build.target, EsTarget::Es2020);
            assert!(!build.dts);
            assert!(!build.minify);
            assert!(!build.splitting);
            assert!(!build.no_treeshake);
            assert!(!build.clean);
            assert!(!build.docs);
            assert!(build.docs_format.is_none());
            assert!(build.docs_dir.is_none());
            assert!(!build.docs_include_internal);
        } else {
            panic!("Expected Build command");
        }
    }

    #[test]
    fn test_build_args_dts_bundle_requires_dts() {
        use clap::Parser;

        // Should fail: --dts-bundle requires --dts
        let result = Cli::try_parse_from(&["joy", "build", "src/index.ts", "--dts-bundle"]);
        assert!(result.is_err());

        // Should succeed: both flags provided
        let result =
            Cli::try_parse_from(&["joy", "build", "src/index.ts", "--dts", "--dts-bundle"]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_dev_args_defaults() {
        use clap::Parser;

        let args = Cli::try_parse_from(&["joy", "dev"]).unwrap();

        if let Command::Dev(dev) = args.command {
            assert_eq!(dev.entry, None); // No default - reads from config
            assert_eq!(dev.port, 3000);
            assert!(!dev.https);
            assert!(!dev.open);
        } else {
            panic!("Expected Dev command");
        }
    }

    #[test]
    fn test_init_args_package_manager_conflicts() {
        use clap::Parser;

        // Should fail: can't use multiple package managers
        let result = Cli::try_parse_from(&["joy", "init", "--use-npm", "--use-yarn"]);
        assert!(result.is_err());

        let result = Cli::try_parse_from(&["joy", "init", "--use-yarn", "--use-pnpm"]);
        assert!(result.is_err());

        // Should succeed: single package manager
        let result = Cli::try_parse_from(&["joy", "init", "--use-pnpm"]);
        assert!(result.is_ok());
    }
}
