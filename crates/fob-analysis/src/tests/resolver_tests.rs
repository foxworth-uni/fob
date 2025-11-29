//! Unit tests for ModuleResolver.

use super::test_helpers::*;
use crate::resolver::ModuleResolver;
use crate::{config::AnalyzerConfig, config::ResolveResult};
use fob_core::test_utils::TestRuntime;
use tempfile::TempDir;

#[tokio::test]
async fn test_resolve_relative_ts_file() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", ""), ("src/utils.ts", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver.resolve("./utils", &from, &runtime).await.unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
    if let ResolveResult::Local(path) = result {
        assert!(path.to_string_lossy().contains("utils.ts"));
    }
}

#[tokio::test]
async fn test_resolve_with_tsx_extension() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.tsx", ""), ("src/Component.tsx", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.tsx");

    let result = resolver
        .resolve("./Component", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
    if let ResolveResult::Local(path) = result {
        assert!(path.to_string_lossy().contains("Component.tsx"));
    }
}

#[tokio::test]
async fn test_resolve_with_js_extension() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.js", ""), ("src/utils.js", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.js");

    let result = resolver.resolve("./utils", &from, &runtime).await.unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
}

#[tokio::test]
async fn test_resolve_index_file() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[("src/index.ts", ""), ("src/components/index.ts", "")],
    );

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("./components", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
    if let ResolveResult::Local(path) = result {
        assert!(path.to_string_lossy().contains("index.ts"));
    }
}

#[tokio::test]
async fn test_resolve_file_already_has_extension() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", ""), ("src/utils.ts", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("./utils.ts", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
}

#[tokio::test]
async fn test_path_alias_resolution() {
    let temp = TempDir::new().unwrap();
    let root = create_project_with_aliases(&temp);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());
    config
        .path_aliases
        .insert("@".to_string(), "./src".to_string());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("@/components/Button", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
    if let ResolveResult::Local(path) = result {
        assert!(path.to_string_lossy().contains("Button"));
    }
}

#[tokio::test]
async fn test_path_alias_without_trailing_slash() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[("src/index.ts", ""), ("src/components/Button.ts", "")],
    );

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());
    config
        .path_aliases
        .insert("@".to_string(), "src".to_string());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("@/components/Button", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
}

#[tokio::test]
async fn test_multiple_path_aliases() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[
            ("src/index.ts", ""),
            ("src/components/Button.ts", ""),
            ("lib/utils.ts", ""),
        ],
    );

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());
    config
        .path_aliases
        .insert("@".to_string(), "./src".to_string());
    config
        .path_aliases
        .insert("~".to_string(), "./lib".to_string());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result1 = resolver
        .resolve("@/components/Button", &from, &runtime)
        .await
        .unwrap();
    let result2 = resolver.resolve("~/utils", &from, &runtime).await.unwrap();

    assert!(matches!(result1, ResolveResult::Local(_)));
    assert!(matches!(result2, ResolveResult::Local(_)));
}

#[tokio::test]
async fn test_external_package_detection() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());
    config.external = vec!["react".to_string()];

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver.resolve("react", &from, &runtime).await.unwrap();

    assert!(matches!(result, ResolveResult::External(_)));
    if let ResolveResult::External(pkg) = result {
        assert_eq!(pkg, "react");
    }
}

#[tokio::test]
async fn test_external_prefix_match() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());
    config.external = vec!["react".to_string()];

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("react-dom", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::External(_)));
}

#[tokio::test]
async fn test_bare_import_is_external() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "")]);

    let config = AnalyzerConfig::default();
    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver.resolve("lodash", &from, &runtime).await.unwrap();

    assert!(matches!(result, ResolveResult::External(_)));
}

#[tokio::test]
async fn test_scoped_package_is_external() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "")]);

    let config = AnalyzerConfig::default();
    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("@types/node", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::External(_)));
}

#[tokio::test]
async fn test_unresolved_file_returns_unresolved() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", "")]);

    let config = AnalyzerConfig::default();
    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let result = resolver
        .resolve("./nonexistent", &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Unresolved(_)));
}

#[tokio::test]
async fn test_resolve_parent_directory() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(
        &temp,
        &[("src/components/Button.ts", ""), ("src/utils.ts", "")],
    );

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/components/Button.ts");

    let result = resolver.resolve("../utils", &from, &runtime).await.unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
}

#[tokio::test]
async fn test_resolve_absolute_path() {
    let temp = TempDir::new().unwrap();
    let root = create_test_project(&temp, &[("src/index.ts", ""), ("src/utils.ts", "")]);

    let mut config = AnalyzerConfig::default();
    config.cwd = Some(root.clone());

    let resolver = ModuleResolver::new(config);
    let runtime = TestRuntime::new(root.clone());
    let from = root.join("src/index.ts");

    let absolute_path = root.join("src/utils.ts");
    let result = resolver
        .resolve(absolute_path.to_str().unwrap(), &from, &runtime)
        .await
        .unwrap();

    assert!(matches!(result, ResolveResult::Local(_)));
}
