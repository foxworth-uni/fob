#[cfg(test)]
mod tests {
    use crate::config::*;
    use std::path::PathBuf;

    #[test]
    fn test_serialization() {
        // Roundtrip serialization
        let config = FobConfig {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Esm,
            out_dir: PathBuf::from("dist"),
            dts: true,
            dts_bundle: Some(true),
            external: vec!["react".to_string()],
            platform: Platform::Browser,
            sourcemap: Some(SourceMapMode::External),
            minify: true,
            target: EsTarget::Es2020,
            global_name: Some("Test".to_string()),
            bundle: true,
            splitting: false,
            no_treeshake: false,
            clean: true,
            cwd: Some(PathBuf::from(".")),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: FobConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.entry, deserialized.entry);

        // Enum serialization (lowercase)
        assert_eq!(serde_json::to_string(&Format::Esm).unwrap(), "\"esm\"");
        assert_eq!(serde_json::to_string(&Platform::Node).unwrap(), "\"node\"");

        // camelCase field names
        let json_val = serde_json::to_value(&config).unwrap();
        assert!(json_val.get("outDir").is_some());
        assert!(json_val.get("dtsBundle").is_some());
        assert!(json_val.get("out_dir").is_none());

        // skip_serializing_if
        let minimal = FobConfig::default_config();
        let json_val = serde_json::to_value(&minimal).unwrap();
        assert!(json_val.get("dtsBundle").is_none());
        assert!(json_val.get("cwd").is_none());
        assert!(json_val.get("docsFormat").is_none());
        assert!(json_val.get("docsDir").is_none());
    }

    #[test]
    fn test_validation() {
        // Empty entry fails
        assert!(FobConfig {
            entry: vec![],
            ..FobConfig::default_config()
        }
        .validate()
        .is_err());

        // IIFE without global_name fails
        assert!(FobConfig {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Iife,
            global_name: None,
            ..FobConfig::default_config()
        }
        .validate()
        .is_err());

        // IIFE with global_name succeeds
        assert!(FobConfig {
            entry: vec!["src/index.ts".to_string()],
            format: Format::Iife,
            global_name: Some("MyLibrary".to_string()),
            ..FobConfig::default_config()
        }
        .validate()
        .is_ok());

        // dts_bundle without dts fails
        assert!(FobConfig {
            entry: vec!["src/index.ts".to_string()],
            dts: false,
            dts_bundle: Some(true),
            ..FobConfig::default_config()
        }
        .validate()
        .is_err());

        // Invalid global names
        for name in ["", "123abc", "my-lib", "my.lib"] {
            assert!(FobConfig {
                entry: vec!["src/index.ts".to_string()],
                format: Format::Iife,
                global_name: Some(name.to_string()),
                ..FobConfig::default_config()
            }
            .validate()
            .is_err());
        }

        // Valid global names
        for name in ["MyLibrary", "_private", "$jquery", "lib123"] {
            assert!(FobConfig {
                entry: vec!["src/index.ts".to_string()],
                format: Format::Iife,
                global_name: Some(name.to_string()),
                ..FobConfig::default_config()
            }
            .validate()
            .is_ok());
        }
    }

    #[test]
    fn test_conversions() {
        use crate::cli::enums::*;

        assert_eq!(Format::from(crate::cli::Format::Esm), Format::Esm);
        assert_eq!(Format::from(crate::cli::Format::Cjs), Format::Cjs);
        assert_eq!(
            Platform::from(crate::cli::Platform::Browser),
            Platform::Browser
        );
        assert_eq!(
            SourceMapMode::from(crate::cli::SourceMapMode::Inline),
            SourceMapMode::Inline
        );
        assert_eq!(
            crate::cli::EsTarget::Es2020,
            EsTarget::Es2020
        );
        assert_eq!(
            crate::cli::EsTarget::Esnext,
            EsTarget::Esnext
        );
    }

    #[test]
    fn test_schema_and_example() {
        assert!(FobConfig::json_schema().is_object());

        let example = FobConfig::example_config();
        let config: FobConfig = serde_json::from_str(&example).unwrap();
        assert!(config.validate().is_ok());
        assert_eq!(config.entry.len(), 2);
        assert!(config.dts && config.minify && config.splitting);
        assert_eq!(config.dts_bundle, Some(true));
    }
}
