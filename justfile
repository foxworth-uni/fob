# Fob Workspace Build Recipes
# Requires: just (https://github.com/casey/just)
# Rust version: 1.91+

# Variables
export RUST_BACKTRACE := "1"
export CARGO_TERM_COLOR := "always"

# Paths
repo_root := justfile_directory()
target_dir := repo_root + "/target"

# Default recipe (runs dev workflow)
default: dev

# List available commands
list:
    @just --list

# =============================================================================
# Core Developer Workflow
# =============================================================================

# The ONE command that verifies everything is good (format + lint + test)
# Run this before committing!
check: format-check lint build-cli test
    @echo "✓ All checks passed! Ready to commit."

# Quick compile check (doesn't run tests/lint)
compile:
    @echo "Checking compilation..."
    @cargo check --workspace --all-features

# Development workflow (format code, then check everything)
dev: format check
    @echo "✓ Development checks complete!"

# Full CI pipeline (format check + lint + test + build)
ci: format-check lint test build
    @echo "✓ All CI checks passed!"

# =============================================================================
# Formatting
# =============================================================================

# Format all code (Rust + TypeScript/JavaScript)
format:
    @echo "Formatting code..."
    @cargo fmt --all
    @pnpm format

# Check formatting without applying (CI-friendly)
format-check:
    @echo "Checking code formatting..."
    @cargo fmt --all -- --check
    @pnpm format:check

# =============================================================================
# Linting
# =============================================================================

# Lint everything (Rust clippy + TypeScript/JavaScript)
lint:
    @echo "Linting code..."
    @cargo clippy --workspace --all-features -- -D warnings
    @pnpm lint

# Lint Rust code only
lint-rust:
    @echo "Linting Rust code..."
    @cargo clippy --workspace --all-features -- -D warnings

# Lint with pedantic warnings
lint-pedantic:
    @echo "Linting Rust code (pedantic)..."
    @cargo clippy --workspace --all-features -- -W clippy::pedantic

# Auto-fix linting issues where possible
lint-fix:
    @echo "Fixing linting issues..."
    @cargo clippy --workspace --all-features --fix --allow-dirty || true
    @pnpm lint --fix || true

# =============================================================================
# Testing
# =============================================================================

# Run all tests (Rust + TypeScript/JavaScript + N-API)
test:
    @echo "Running tests..."
    @cargo test --workspace
    @just build-cli
    @pnpm test
    @just test-napi

# Run tests with verbose output
test-verbose:
    @echo "Running tests (verbose)..."
    @cargo test --workspace -- --nocapture

# Watch tests (requires cargo-watch)
test-watch:
    @echo "Watching tests..."
    @cargo watch -x test

# Test specific package
test-package package:
    @cargo test --package {{package}}

# Test Rust only
test-rust:
    @echo "Running Rust tests..."
    @cargo test --workspace

# =============================================================================
# Building
# =============================================================================

# Build everything needed for development
build: build-native
    @echo "✓ Build complete!"

# Build everything in release mode
build-release: build-native-release build-napi-release
    @echo "✓ Release build complete!"

# Build native crates
build-native:
    @echo "Building native crates..."
    @cargo build --workspace

# Build native crates in release mode
build-native-release:
    @echo "Building native crates (release)..."
    @cargo build --workspace --release

# Build N-API bindings
build-napi:
    @echo "Building N-API bindings..."
    @cd crates/fob-native && pnpm build

# Build N-API bindings (release)
build-napi-release:
    @echo "Building N-API bindings (release)..."
    @cd crates/fob-native && cargo build --release

# Build N-API bindings for specific platform
build-napi-platform platform:
    @echo "Building N-API for {{platform}}..."
    @cd crates/fob-native && cargo build --target {{platform}} --release

# Build N-API bindings (debug) using napi CLI
build-napi-debug:
    @echo "Building N-API bindings (debug)..."
    @cd crates/fob-native && pnpm build:debug

# Build N-API and sync to node_modules (for local development)
build-napi-sync: build-napi-debug
    @echo "✓ N-API binary built and synced!"

# Run JavaScript integration tests for N-API bindings
test-napi:
    @echo "Running N-API JavaScript tests..."
    @cd fixtures/napi-local-test && pnpm test

# Run simple N-API tests only
test-napi-simple:
    @cd fixtures/napi-local-test && pnpm test:simple

# Run advanced N-API tests only
test-napi-advanced:
    @cd fixtures/napi-local-test && pnpm test:advanced

# Run error handling N-API tests only
test-napi-errors:
    @cd fixtures/napi-local-test && pnpm test:errors

# Full N-API development workflow (build + sync + test)
dev-napi: build-napi-sync test-napi
    @echo "✓ N-API development workflow complete!"

# Verify N-API bindings work correctly (build release + test)
verify-napi: build-napi-release
    @echo "Verifying N-API bindings..."
    @cd fixtures/napi-local-test && pnpm test
    @echo "✓ N-API verification passed!"

# Build CLI
build-cli:
    @cargo build --package fob-cli

# Build CLI (release)
build-cli-release:
    @cargo build --package fob-cli --release

# =============================================================================
# Compilation Checks
# =============================================================================

# Check specific package compiles
compile-package package:
    @cargo check --package {{package}} --all-features

# =============================================================================
# Package-Specific Commands
# =============================================================================

# fob
compile-core:
    @cargo check --package fob --all-features

test-core:
    @cargo test --package fob --all-features

docs-core:
    @cargo doc --package fob --all-features --no-deps --open

# fob-cli
test-cli:
    @cargo test --package fob-cli

run-cli *args:
    @cargo run --package fob-cli -- {{args}}

# fob-config
test-config:
    @cargo test --package fob-config --all-features

# fob-gen
test-gen:
    @cargo test --package fob-gen --all-features

compile-gen:
    @cargo check --package fob-gen --all-features

docs-gen:
    @cargo doc --package fob-gen --all-features --no-deps --open

# fob-browser-test
test-browser:
    @cargo test --package fob-browser-test

compile-browser:
    @cargo check --package fob-browser-test

# Plugins
test-plugins: test-plugin-css test-plugin-tailwind
    @echo "✓ All plugin tests passed!"

compile-plugins: compile-plugin-css compile-plugin-tailwind
    @echo "✓ All plugins compile!"

test-plugin-css:
    @cargo test --package fob-plugin-css --all-features

compile-plugin-css:
    @cargo check --package fob-plugin-css --all-features

test-plugin-tailwind:
    @cargo test --package fob-plugin-tailwind --all-features

compile-plugin-tailwind:
    @cargo check --package fob-plugin-tailwind --all-features


# =============================================================================
# Installation
# =============================================================================

# Install CLI to cargo bin
install-cli:
    @cargo install --path crates/fob-cli

# Install build tools (cargo-watch, cargo-release, etc.)
setup-tools:
    @echo "Installing build tools..."
    @command -v cargo-watch || cargo install cargo-watch
    @command -v cargo-audit || cargo install cargo-audit
    @command -v cargo-release || cargo install cargo-release
    @echo "✓ Build tools installed"

# =============================================================================
# Cleanup
# =============================================================================

# Clean all build artifacts
clean:
    @echo "Cleaning build artifacts..."
    @cargo clean
    @pnpm clean || true
    @echo "✓ Clean complete"

# Clean WASM build artifacts only
clean-wasm:
    @echo "Cleaning WASM artifacts..."
    @rm -rf crates/fob-mdx-wasm/pkg crates/fob-mdx-wasm/pkg-node
    @rm -rf packages/fob-mdx-wasm/pkg packages/fob-mdx-wasm/pkg-node
    @rm -rf packages/fob-mdx-wasm/dist
    @echo "✓ WASM artifacts cleaned"

# =============================================================================
# Verification & Quality
# =============================================================================

# Verify all builds work correctly
verify: verify-native verify-tests
    @echo "✓ All verification checks passed!"

# Verify native builds
verify-native:
    @echo "Verifying native builds..."
    @cargo build --workspace
    @cargo test --workspace
    @echo "✓ Native verification passed!"

# Verify all tests pass
verify-tests:
    @echo "Running comprehensive test suite..."
    @cargo test --workspace --all-features
    @pnpm test
    @echo "✓ All tests passed!"

# Security audit (requires cargo-audit)
audit:
    @echo "Running security audit..."
    @cargo audit || (echo "Install cargo-audit: cargo install cargo-audit" && exit 1)

# Check for outdated dependencies
outdated:
    @echo "Checking for outdated dependencies..."
    @cargo outdated || (echo "Install cargo-outdated: cargo install cargo-outdated" && exit 1)

# =============================================================================
# Documentation
# =============================================================================

# Build all documentation
docs:
    @echo "Building documentation..."
    @cargo doc --workspace --all-features --no-deps

# Build and open documentation
docs-open:
    @echo "Building and opening documentation..."
    @cargo doc --workspace --all-features --no-deps --open

# =============================================================================
# Releasing
# =============================================================================

# Preview a release (dry-run)
release-dry level:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-release >/dev/null || { echo "Install: cargo install cargo-release"; exit 1; }
    echo "Previewing {{level}} release..."
    cargo release {{level}} --workspace --no-verify


publish:
    #!/usr/bin/env bash
    set -euo pipefail
    command -v cargo-release >/dev/null || { echo "Install: cargo install cargo-release"; exit 1; }
    echo "Publishing unpublished crates..."
    cargo release publish --workspace --execute --no-verify

# Release a new version (interactive)
release:
    #!/usr/bin/env bash
    set -euo pipefail

    command -v gum >/dev/null || { echo "Install gum: brew install gum"; exit 1; }
    command -v cargo-release >/dev/null || { echo "Install: cargo install cargo-release"; exit 1; }

    LEVEL=$(gum choose "patch" "minor" "major")

    echo "Previewing $LEVEL release..."
    cargo release $LEVEL --workspace --no-verify

    gum confirm "Execute this release?" || exit 0

    # Bump versions only (no commit, no tag - we handle those manually)
    echo "Bumping versions..."
    cargo release $LEVEL --workspace --execute --no-confirm --no-verify --no-publish --no-tag

    # Extract version
    VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)

    # Sync npm version
    node -e "
      const fs = require('fs');
      const pkg = JSON.parse(fs.readFileSync('crates/fob-native/package.json'));
      pkg.version = '$VERSION';
      fs.writeFileSync('crates/fob-native/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "

    # Commit all changes
    git add -A
    git commit -m "chore: release v$VERSION"

    # Create annotated tag
    git tag -a "v$VERSION" -m "Release v$VERSION"

    # Push
    gum confirm "Push to origin?" && git push && git push --tags

    # Publish
    gum confirm "Publish to crates.io?" || exit 0
    echo "Publishing to crates.io..."
    cargo release publish --workspace --execute --no-verify

    echo "Done!"

# =============================================================================
# Development Tools
# =============================================================================

# Watch for changes and run checks (requires cargo-watch)
watch:
    @echo "Watching for changes..."
    @cargo watch -x check -x test

# Watch specific package
watch-package package:
    @echo "Watching {{package}}..."
    @cargo watch -x "check --package {{package}}" -x "test --package {{package}}"


# =============================================================================
# Utility Recipes
# =============================================================================

# Show build configuration
info:
    @echo "Fob Workspace Configuration"
    @echo "============================"
    @echo "Rust version: $(rustc --version)"
    @echo "Cargo version: $(cargo --version)"
    @echo "Repository root: {{repo_root}}"
    @echo "Target directory: {{target_dir}}"
    @echo ""
    @echo "Installed targets:"
    @rustup target list | grep installed

# Count lines of code
loc:
    @echo "Lines of code:"
    @find crates -name "*.rs" -not -path "*/target/*" | xargs wc -l | tail -1

# Show dependency tree for a package
deps package:
    @cargo tree --package {{package}}

