#[cfg(test)]
mod tests {
    use crate::builders::asset_resolver::{resolve_asset, validate_asset_size};
    use crate::test_utils::TestRuntime;
    use crate::builders::asset_resolver::security::find_monorepo_root;
    use std::fs;
    use tempfile::TempDir;

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_resolve_relative() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create test structure
        let src_dir = cwd.join("src");
        let assets_dir = src_dir.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();

        let asset_file = assets_dir.join("test.wasm");
        fs::write(&asset_file, b"test").unwrap();

        let referrer = src_dir.join("index.js");
        fs::write(&referrer, b"").unwrap();

        // Resolve relative path
        let resolved = resolve_asset("./assets/test.wasm", &referrer, &cwd, &runtime).await.unwrap();
        assert_eq!(resolved.canonicalize().unwrap(), asset_file.canonicalize().unwrap());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_resolve_from_node_modules() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create node_modules structure
        let nm_dir = cwd.join("node_modules/@test/pkg/wasm");
        fs::create_dir_all(&nm_dir).unwrap();

        let asset_file = nm_dir.join("file.wasm");
        fs::write(&asset_file, b"test").unwrap();

        let referrer = cwd.join("src/index.js");
        fs::create_dir_all(referrer.parent().unwrap()).unwrap();
        fs::write(&referrer, b"").unwrap();

        // Resolve from node_modules
        let resolved = resolve_asset("@test/pkg/wasm/file.wasm", &referrer, &cwd, &runtime).await.unwrap();
        assert_eq!(resolved.canonicalize().unwrap(), asset_file.canonicalize().unwrap());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_security_directory_traversal() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create file outside project
        let outside = temp.path().parent().unwrap().join("outside.wasm");
        fs::write(&outside, b"test").unwrap();

        let referrer = cwd.join("src/index.js");
        fs::create_dir_all(referrer.parent().unwrap()).unwrap();
        fs::write(&referrer, b"").unwrap();

        // Try to resolve path that escapes project
        let result = resolve_asset("../../outside.wasm", &referrer, &cwd, &runtime).await;
        assert!(result.is_err());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_validate_asset_size() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());
        let small_file = cwd.join("small.wasm");
        fs::write(&small_file, vec![0u8; 1024]).unwrap(); // 1KB

        // Should succeed with default limit
        let size = validate_asset_size(&small_file, None, &runtime).await.unwrap();
        assert_eq!(size, 1024);

        // Should fail with tiny limit
        let result = validate_asset_size(&small_file, Some(512), &runtime).await;
        assert!(result.is_err());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_monorepo_asset_access() {
        let temp = TempDir::new().unwrap();
        let monorepo_root = temp.path().to_path_buf();

        // Create monorepo structure with pnpm-workspace.yaml
        fs::write(
            monorepo_root.join("pnpm-workspace.yaml"),
            "packages:\n  - 'packages/*'\n",
        ).unwrap();

        // Create workspace packages
        let pkg_a = monorepo_root.join("packages/pkg-a/src");
        let pkg_b = monorepo_root.join("packages/pkg-b/assets");
        fs::create_dir_all(&pkg_a).unwrap();
        fs::create_dir_all(&pkg_b).unwrap();

        // Asset in package B referenced from package A
        let asset_file = pkg_b.join("logo.png");
        fs::write(&asset_file, b"test").unwrap();

        let referrer = pkg_a.join("index.js");
        fs::write(&referrer, b"").unwrap();

        // Set cwd to package A (where the bundling happens)
        let cwd = monorepo_root.join("packages/pkg-a");
        let runtime = TestRuntime::new(cwd.clone());

        // Should succeed - asset is in same monorepo
        let resolved = resolve_asset("../../pkg-b/assets/logo.png", &referrer, &cwd, &runtime).await;
        assert!(resolved.is_ok(), "Failed to resolve monorepo asset: {:?}", resolved.err());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_find_monorepo_root_pnpm() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let runtime = TestRuntime::new(root.clone());

        // Create pnpm workspace
        fs::write(root.join("pnpm-workspace.yaml"), "packages:\n  - 'packages/*'\n").unwrap();

        let pkg_dir = root.join("packages/pkg-a");
        fs::create_dir_all(&pkg_dir).unwrap();

        // Should find root from package directory
        let found_root = find_monorepo_root(&pkg_dir, &runtime).await;
        assert!(found_root.is_some());
        assert_eq!(found_root.unwrap().canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_find_monorepo_root_npm_workspaces() {
        let temp = TempDir::new().unwrap();
        let root = temp.path().to_path_buf();
        let runtime = TestRuntime::new(root.clone());

        // Create npm/yarn workspace
        fs::write(
            root.join("package.json"),
            r#"{"name": "monorepo", "workspaces": ["packages/*"]}"#,
        ).unwrap();

        let pkg_dir = root.join("packages/pkg-a");
        fs::create_dir_all(&pkg_dir).unwrap();

        // Should find root from package directory
        let found_root = find_monorepo_root(&pkg_dir, &runtime).await;
        assert!(found_root.is_some());
        assert_eq!(found_root.unwrap().canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_bare_filename_resolution() {
        // Test wasm-bindgen pattern: new URL('file.wasm', import.meta.url)
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create directory structure similar to wasm-bindgen output
        let wasm_dir = cwd.join("node_modules/@pkg/wasm/web");
        fs::create_dir_all(&wasm_dir).unwrap();

        // Create the WASM file and JS file in same directory
        let wasm_file = wasm_dir.join("pkg_bg.wasm");
        fs::write(&wasm_file, b"wasm").unwrap();

        let js_file = wasm_dir.join("pkg.js");
        fs::write(&js_file, b"js").unwrap();

        // Resolve bare filename from JS file (no ./ prefix)
        let resolved = resolve_asset("pkg_bg.wasm", &js_file, &cwd, &runtime).await;
        assert!(resolved.is_ok(), "Failed to resolve bare filename: {:?}", resolved.err());

        let resolved_path = resolved.unwrap();
        assert_eq!(
            resolved_path.canonicalize().unwrap(),
            wasm_file.canonicalize().unwrap(),
            "Bare filename should resolve to file in same directory"
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_package_path_vs_bare_filename() {
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        // Create test structure
        let src_dir = cwd.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let pkg_dir = cwd.join("node_modules/pkg/assets");
        fs::create_dir_all(&pkg_dir).unwrap();

        // Create asset in node_modules
        let pkg_asset = pkg_dir.join("file.wasm");
        fs::write(&pkg_asset, b"pkg").unwrap();

        let referrer = src_dir.join("index.js");
        fs::write(&referrer, b"").unwrap();

        // Package path with slashes should resolve from node_modules
        let resolved = resolve_asset("pkg/assets/file.wasm", &referrer, &cwd, &runtime).await;
        assert!(resolved.is_ok(), "Package path should resolve: {:?}", resolved.err());
        assert_eq!(
            resolved.unwrap().canonicalize().unwrap(),
            pkg_asset.canonicalize().unwrap()
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[tokio::test]
    async fn test_scoped_package_bare_filename() {
        // Test @scope/file.wasm pattern (single slash)
        let temp = TempDir::new().unwrap();
        let cwd = temp.path().to_path_buf();
        let runtime = TestRuntime::new(cwd.clone());

        let scope_dir = cwd.join("node_modules/@scope");
        fs::create_dir_all(&scope_dir).unwrap();

        let asset = scope_dir.join("file.wasm");
        fs::write(&asset, b"scoped").unwrap();

        let referrer = cwd.join("src/index.js");
        fs::create_dir_all(referrer.parent().unwrap()).unwrap();
        fs::write(&referrer, b"").unwrap();

        // @scope/file.wasm should look in node_modules/@scope/
        let resolved = resolve_asset("@scope/file.wasm", &referrer, &cwd, &runtime).await;
        assert!(resolved.is_ok(), "Scoped package should resolve: {:?}", resolved.err());
    }
}

