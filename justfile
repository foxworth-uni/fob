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
check: format-check lint test
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
test-plugins: test-plugin-css test-plugin-tailwind test-plugin-vue test-plugin-svelte test-plugin-astro
    @echo "✓ All plugin tests passed!"

compile-plugins: compile-plugin-css compile-plugin-tailwind compile-plugin-vue compile-plugin-svelte compile-plugin-astro
    @echo "✓ All plugins compile!"

test-plugin-css:
    @cargo test --package fob-plugin-css --all-features

compile-plugin-css:
    @cargo check --package fob-plugin-css --all-features

test-plugin-tailwind:
    @cargo test --package fob-plugin-tailwind --all-features

compile-plugin-tailwind:
    @cargo check --package fob-plugin-tailwind --all-features

test-plugin-vue:
    @cargo test --package fob-plugin-vue --all-features

compile-plugin-vue:
    @cargo check --package fob-plugin-vue --all-features

test-plugin-svelte:
    @cargo test --package fob-plugin-svelte --all-features

compile-plugin-svelte:
    @cargo check --package fob-plugin-svelte --all-features

test-plugin-astro:
    @cargo test --package fob-plugin-astro --all-features

compile-plugin-astro:
    @cargo check --package fob-plugin-astro --all-features

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

# Bootstrap publish - one crate at a time in dependency order
# Safe to re-run: cargo publish skips already-published versions
publish-bootstrap:
    #!/usr/bin/env bash
    set -euo pipefail

    CRATES=(
        fob-browser-test
        fob-config
        fob-gen
        fob-graph
        fob-bundler
        fob-cli
        fob-plugin-css
        fob-plugin-vue
        fob-plugin-svelte
        fob-plugin-astro
    )

    for crate in "${CRATES[@]}"; do
        echo "Publishing $crate..."
        cargo publish -p "$crate" --no-verify || echo "Skipped $crate (may already exist)"
        sleep 2  # Be nice to crates.io
    done

    echo "Done!"

# Publish using cargo-release (use after bootstrap, when crates exist)
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

    # Check tools
    command -v gum >/dev/null || { echo "Install gum: brew install gum"; exit 1; }
    command -v cargo-release >/dev/null || { echo "Install: cargo install cargo-release"; exit 1; }

    # Pick level
    LEVEL=$(gum choose "patch" "minor" "major")

    # Preview
    echo "Previewing $LEVEL release..."
    cargo release $LEVEL --workspace --no-verify

    # Confirm
    gum confirm "Execute this release?" || exit 0

    # Step 1: Bump versions (no publish yet)
    echo "Bumping versions..."
    cargo release $LEVEL --workspace --execute --no-confirm --no-verify --no-publish

    # Sync npm version (read from workspace Cargo.toml)
    VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
    node -e "
      const fs = require('fs');
      const pkg = JSON.parse(fs.readFileSync('crates/fob-native/package.json'));
      pkg.version = '$VERSION';
      fs.writeFileSync('crates/fob-native/package.json', JSON.stringify(pkg, null, 2) + '\n');
    "
    git add crates/fob-native/package.json
    git commit --amend --no-edit

    # Push
    gum confirm "Push to origin?" && git push && git push --tags

    # Step 2: Publish (auto-skips already published, safe to re-run)
    gum confirm "Publish to crates.io?" || exit 0
    echo "Publishing to crates.io..."
    cargo release publish --workspace --execute --no-verify

    echo "Done! If rate-limited, run 'just publish' to continue."

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
# Examples
# =============================================================================

# List all available examples
examples:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "📦 Available Examples"
    echo ""
    echo "Rust Examples:"
    for dir in examples/rust/*/; do
        if [ -f "$dir/Cargo.toml" ]; then
            name=$(basename "$dir")
            desc=$(grep -m1 "^# " "$dir/README.md" 2>/dev/null | sed 's/^# //' || echo "")
            if [ -n "$desc" ]; then
                printf "  %-30s %s\n" "$name" "$desc"
            else
                printf "  %s\n" "$name"
            fi
        fi
    done
    echo ""
    echo "JavaScript Examples:"
    for dir in examples/js/*/; do
        if [ -f "$dir/package.json" ]; then
            name=$(basename "$dir")
            desc=$(grep -m1 '"description"' "$dir/package.json" 2>/dev/null | sed 's/.*"description": "\(.*\)".*/\1/' || echo "")
            if [ -n "$desc" ]; then
                printf "  %-30s %s\n" "$name" "$desc"
            else
                printf "  %s\n" "$name"
            fi
        fi
    done
    echo ""
    echo "💡 Quick Start:"
    echo "  just example rust/basic-bundler        # Start here!"
    echo "  just example rust/advanced-bundler     # Production patterns"
    echo "  just example rust/component-library    # React components"
    echo "  just example rust/meta-framework       # Framework building"
    echo ""
    echo "Run any example:"
    echo "  just example rust/<name>"
    echo "  just example js/<name>"

# Run a specific example
example name:
    #!/usr/bin/env bash
    set -euo pipefail
    NAME="{{name}}"
    if [[ "$NAME" == rust/* ]]; then
        example_name="${NAME#rust/}"
        example_dir="examples/rust/$example_name"
        if [ ! -d "$example_dir" ]; then
            echo "❌ Rust example '$example_name' not found"
            echo "Run 'just examples' to see available examples"
            exit 1
        fi
        echo "🦀 Running Rust example: $example_name"
        cd "$example_dir" && cargo run
    elif [[ "$NAME" == js/* ]]; then
        example_name="${NAME#js/}"
        example_dir="examples/js/$example_name"
        if [ ! -d "$example_dir" ]; then
            echo "❌ JavaScript example '$example_name' not found"
            echo "Run 'just examples' to see available examples"
            exit 1
        fi
        echo "📦 Running JavaScript example: $example_name"
        cd "$example_dir" && npm run start 2>/dev/null || npm run dev 2>/dev/null || node src/index.js
    else
        echo "❌ Invalid example format. Use 'rust/<name>' or 'js/<name>'"
        echo "Run 'just examples' to see available examples"
        exit 1
    fi

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

