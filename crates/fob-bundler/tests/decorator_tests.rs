/// Modern decorator transformation tests
#[cfg(not(target_family = "wasm"))]
mod decorator_tests {
    use fob_bundler::BuildOptions;
    use fob_graph::runtime::native::NativeRuntime;
    use std::env;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Get the path to test fixtures relative to the workspace root
    fn fixture_path(relative: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push(relative);
        path
    }

    #[tokio::test]
    async fn test_modern_decorators() -> fob_bundler::Result<()> {
        let fixture = fixture_path("fixtures/decorators/modern.ts");

        // Build with modern decorator support enabled
        let result = BuildOptions::new(&fixture)
            .bundle(false) // Library mode
            .decorators(true)
            .cwd(env!("CARGO_MANIFEST_DIR"))
            .runtime(Arc::new(NativeRuntime))
            .build()
            .await?;

        // Verify successful build
        let bundle = result.output.as_single().expect("single bundle");

        // Basic smoke test: verify output was generated
        assert!(!bundle.assets.is_empty(), "Should generate output assets");

        // Get the first chunk to verify it contains code
        let chunk = result
            .chunks()
            .next()
            .expect("Should have at least one chunk");
        assert!(chunk.code.len() > 0, "Generated code should not be empty");

        // Verify the transformed code contains our class and method
        assert!(chunk.code.contains("MyClass"), "Should contain MyClass");
        assert!(chunk.code.contains("greet"), "Should contain greet method");

        Ok(())
    }
}
