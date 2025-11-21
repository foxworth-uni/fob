# Fob Workspace Build Recipes
# Requires: just (https://github.com/casey/just)
# Rust version: 1.77+

# Variables
export RUST_BACKTRACE := "1"
export CARGO_TERM_COLOR := "always"

# Paths
repo_root := justfile_directory()
target_dir := repo_root + "/target"
wasm_wasi_target := "wasm32-wasip1"

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

# Full CI pipeline (format check + lint + test + build + wasm checks)
ci: format-check lint test build check-std-fs lint-wasm compile-wasm wasm-size
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

# Lint WASM-specific code
lint-wasm:
    @echo "Linting WASM code..."
    @cargo clippy --package fob --target {{wasm_wasi_target}} --all-features -- -D warnings
    @cargo clippy --package fob-wasm --target {{wasm_wasi_target}} -- -D warnings

# =============================================================================
# Testing
# =============================================================================

# Run all tests (Rust + TypeScript/JavaScript)
test:
    @echo "Running tests..."
    @cargo test --workspace
    @pnpm test

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
build: build-native build-wasm
    @echo "✓ Build complete!"

# Build everything in release mode
build-release: build-native-release build-wasm-release build-napi-release
    @echo "✓ Release build complete!"

# Build native crates
build-native:
    @echo "Building native crates..."
    @cargo build --workspace --exclude fob-wasm

# Build native crates in release mode
build-native-release:
    @echo "Building native crates (release)..."
    @cargo build --workspace --exclude fob-wasm --release

# Build WASM (defaults to release)
build-wasm: build-wasm-release

# Build WASM in release mode
build-wasm-release:
    @echo "Building WASM (release)..."
    @cd crates/fob-wasm && ./build.sh release

# Build WASM in dev mode (faster, unoptimized)
build-wasm-dev:
    @echo "Building WASM (dev)..."
    @cd crates/fob-wasm && ./build.sh dev

# Setup WASM tooling (wasm-tools, jco, target)
setup-wasm:
    @echo "Setting up WASM tooling..."
    @command -v wasm-tools >/dev/null || cargo install wasm-tools
    @command -v jco >/dev/null || npm install -g @bytecodealliance/jco@1.8.0
    @rustup target add wasm32-wasip1
    @echo "✓ WASM tooling ready"

# Build N-API bindings
build-napi:
    @echo "Building N-API bindings..."
    @cd crates/fob-native && cargo build

# Build N-API bindings (release)
build-napi-release:
    @echo "Building N-API bindings (release)..."
    @cd crates/fob-native && cargo build --release

# Build N-API bindings for specific platform
build-napi-platform platform:
    @echo "Building N-API for {{platform}}..."
    @cd crates/fob-native && cargo build --target {{platform}} --release

# Build TypeScript bundler package
build-bundler-ts: _copy-native
    @echo "Building TypeScript bundler package..."
    @cd packages/fob-bundler && pnpm install
    @cd packages/fob-bundler && pnpm build
    @echo "✓ TypeScript bundler built"

# Full bundler build (development)
build-bundler: build-napi build-bundler-ts
    @echo "✓ Bundler development build complete!"

# Full bundler build (release)
build-bundler-release: build-napi-release build-bundler-ts
    @echo "✓ Bundler release build complete!"

# Build CLI
build-cli:
    @cargo build --package fob-cli

# Build CLI (release)
build-cli-release:
    @cargo build --package fob-cli --release

# Build Gumbo CLI
build-gumbo:
    @cargo build --package gumbo-cli

# Build Gumbo CLI (release)
build-gumbo-release:
    @cargo build --package gumbo-cli --release

# =============================================================================
# Compilation Checks
# =============================================================================

# Check WASM crate compiles
compile-wasm:
    @echo "Checking WASM compilation..."
    @cargo check --package fob-wasm --target {{wasm_wasi_target}}

# Check for accidental std::fs usage (WASM compatibility)
check-std-fs:
    @echo "Checking for disallowed std::fs usage..."
    @! grep -r "std::fs" crates/fob/src \
        --include="*.rs" \
        --exclude-dir=target \
        | grep -v "test_utils.rs" \
        | grep -v "native_runtime.rs" \
        | grep -v "#\[cfg(test)\]" \
        | grep -v "//.*std::fs" \
        || (echo "❌ Found disallowed std::fs usage! Use Runtime trait instead." && exit 1)
    @echo "✓ No disallowed std::fs usage found"

# Check specific package compiles
compile-package package:
    @cargo check --package {{package}} --all-features

# =============================================================================
# WASM Utilities
# =============================================================================

# Run WASM-specific tests (native host, WASM target tests)
test-wasm:
    @echo "Running WASM tests..."
    @cargo test --package fob-wasm

# Show WASM bundle sizes
wasm-size:
    @echo "📦 WASM Bundle Sizes:"
    @echo ""
    @echo "Core WASM:"
    @ls -lh crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm 2>/dev/null | awk '{print "  " $$5 "\t" $$9}' || \
        ls -lh crates/fob-wasm/pkg/debug/fob_bundler_wasm.wasm 2>/dev/null | awk '{print "  " $$5 "\t" $$9}' || \
        echo "  (No build found - run 'just build-wasm' first)"
    @echo ""
    @echo "Component Model:"
    @ls -lh crates/fob-wasm/pkg/release/fob_bundler.component.wasm 2>/dev/null | awk '{print "  " $$5 "\t" $$9}' || \
        echo "  (Component not built)"
    @echo ""
    @echo "Edge Package:"
    @ls -lh packages/fob-edge/wasm/bundler/*.wasm 2>/dev/null | awk '{print "  " $$5 "\t" $$9}' || \
        echo "  (Not copied to edge package yet)"

# Check WASM size against edge runtime limits
wasm-size-check:
    @echo "📦 Checking WASM size against edge runtime limits..."
    @echo ""
    @if [ -f "crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm" ]; then \
        SIZE=$$(stat -f%z "crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm" 2>/dev/null || stat -c%s "crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm" 2>/dev/null); \
        SIZE_MB=$$(echo "scale=2; $$SIZE / 1024 / 1024" | bc); \
        echo "  Size: $${SIZE_MB} MB"; \
        if [ $$(echo "$$SIZE_MB < 3" | bc) -eq 1 ]; then \
            echo "  ✓ Under 3MB (Cloudflare Workers Free Tier)"; \
        elif [ $$(echo "$$SIZE_MB < 10" | bc) -eq 1 ]; then \
            echo "  ⚠ Under 10MB (Cloudflare Workers Paid Tier)"; \
        else \
            echo "  ✗ Over 10MB (too large for most edge runtimes)"; \
            exit 1; \
        fi; \
    else \
        echo "  ⚠ WASM file not found"; \
        echo "  Run: just build-wasm"; \
        exit 1; \
    fi

# Test WASM binary with wasmtime runtime
wasm-run:
    @echo "Testing WASM with wasmtime..."
    @wasmtime --version || (echo "Install wasmtime: curl https://wasmtime.dev/install.sh -sSf | bash" && exit 1)
    @test -f crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm || (echo "Build first: just build-wasm" && exit 1)
    @wasmtime crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm

# Verify WASM builds work
wasm-verify: compile-wasm
    @echo "✓ WASM verification passed!"

# Verify WASM compatibility enforcement
wasm-verify-compat: check-std-fs lint-wasm compile-wasm
    @echo "✓ WASM compatibility verification passed!"

# Full WASM development workflow (setup + build + test + size check)
wasm-dev: setup-wasm build-wasm-dev test-wasm wasm-size
    @echo "✓ WASM development workflow complete!"

# =============================================================================
# Package-Specific Commands
# =============================================================================

# fob-core
compile-core:
    @cargo check --package fob --all-features

test-core:
    @cargo test --package fob --all-features

test-core-feature feature:
    @cargo test --package fob --features {{feature}}

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
test-plugins: test-plugin-css test-plugin-mdx test-plugin-tailwind
    @echo "✓ All plugin tests passed!"

compile-plugins: compile-plugin-css compile-plugin-mdx compile-plugin-tailwind
    @echo "✓ All plugins compile!"

test-plugin-css:
    @cargo test --package fob-plugin-css --all-features

compile-plugin-css:
    @cargo check --package fob-plugin-css --all-features

test-plugin-mdx:
    @cargo test --package fob-plugin-mdx --all-features

compile-plugin-mdx:
    @cargo check --package fob-plugin-mdx --all-features

test-plugin-tailwind:
    @cargo test --package fob-plugin-tailwind --all-features

compile-plugin-tailwind:
    @cargo check --package fob-plugin-tailwind --all-features

# Gumbo
test-gumbo:
    @cargo test --package gumbo-core
    @cargo test --package gumbo-cli

# =============================================================================
# Installation
# =============================================================================

# Install CLI to cargo bin
install-cli:
    @cargo install --path crates/fob-cli

# Install Gumbo CLI
install-gumbo:
    @cargo install --path crates/gumbo-cli

# Install Rust targets for WASM builds
setup-targets:
    @echo "Installing Rust WASM targets..."
    @rustup target add {{wasm_wasi_target}}
    @echo "✓ WASM targets installed"

# Install build tools (wasm-pack, cargo-watch, etc.)
setup-tools:
    @echo "Installing build tools..."
    @command -v wasm-pack || cargo install wasm-pack
    @command -v cargo-watch || cargo install cargo-watch
    @command -v cargo-audit || cargo install cargo-audit
    @command -v wasmtime || echo "Install wasmtime: curl https://wasmtime.dev/install.sh -sSf | bash"
    @echo "✓ Build tools installed"

# =============================================================================
# Cleanup
# =============================================================================

# Clean all build artifacts
clean:
    @echo "Cleaning build artifacts..."
    @cargo clean
    @rm -rf crates/fob-wasm/pkg
    @pnpm clean || true
    @echo "✓ Clean complete"

# Clean WASM artifacts
clean-wasm:
    @rm -rf crates/fob-wasm/pkg
    @rm -rf target/{{wasm_wasi_target}}
    @rm -rf packages/fob-edge/wasm/bundler
    @echo "✓ WASM artifacts cleaned"

# =============================================================================
# Copy Utilities
# =============================================================================

# Copy native binary to TypeScript package (Internal)
_copy-native:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Copying native binary to fob-bundler package..."

    # Detect platform and set extension
    case "$(uname -s)" in
        Darwin*) EXT="dylib" ;;
        Linux*)  EXT="so" ;;
        MINGW*|MSYS*|CYGWIN*) EXT="dll" ;;
        *) echo "❌ Unknown platform"; exit 1 ;;
    esac

    # Copy from target/debug or target/release
    if [ -f "target/release/libfob_native.$EXT" ]; then
        echo "  Found release build"
        cp "target/release/libfob_native.$EXT" packages/fob-bundler/index.node
    elif [ -f "target/debug/libfob_native.$EXT" ]; then
        echo "  Found debug build"
        cp "target/debug/libfob_native.$EXT" packages/fob-bundler/index.node
    else
        echo "❌ Native library not found. Run 'just build:napi' or 'just build:napi:release' first"
        exit 1
    fi

    echo "✓ Native binary copied to packages/fob-bundler/index.node"

# =============================================================================
# Verification & Quality
# =============================================================================

# Verify all builds work correctly
verify: verify-native wasm-verify verify-tests
    @echo "✓ All verification checks passed!"

# Verify native builds
verify-native:
    @echo "Verifying native builds..."
    @cargo build --workspace --exclude fob-wasm
    @cargo test --workspace --exclude fob-wasm
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
# Benchmarking & Performance
# =============================================================================

# Run benchmarks (if any exist)
bench:
    @echo "Running benchmarks..."
    @cargo bench --workspace

# Run benchmarks for specific package
bench-package package:
    @cargo bench --package {{package}}

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
# Release Management
# =============================================================================

# Interactive version bump and git tag (mirrors danny workflow)
tag:
    #!/usr/bin/env bash
    set -euo pipefail

    # Require gum for interactive prompts
    if ! command -v gum &> /dev/null; then
        echo "❌ gum is required. Install with: brew install gum"
        exit 1
    fi

    # Get current workspace version from root Cargo.toml
    CURRENT_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
    echo "📦 Current version: $CURRENT_VERSION"

    # Choose version bump
    BUMP=$(gum choose "patch" "minor" "major" "custom" --header "Select version increment")

    if [ "$BUMP" = "custom" ]; then
        NEW_VERSION=$(gum input --placeholder "e.g. 0.2.0" --value "$CURRENT_VERSION")
    else
        IFS='.' read -r -a PARTS <<< "$CURRENT_VERSION"
        MAJOR="${PARTS[0]}"
        MINOR="${PARTS[1]}"
        PATCH="${PARTS[2]}"

        case "$BUMP" in
            patch) PATCH=$((PATCH + 1)) ;;
            minor) MINOR=$((MINOR + 1)); PATCH=0 ;;
            major) MAJOR=$((MAJOR + 1)); MINOR=0; PATCH=0 ;;
        esac
        NEW_VERSION="$MAJOR.$MINOR.$PATCH"
    fi

    echo "🚀 Preparing to bump: $CURRENT_VERSION -> $NEW_VERSION"
    if ! gum confirm "Proceed?"; then
        echo "Cancelled"
        exit 0
    fi

    # Update workspace Cargo.toml
    sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" Cargo.toml

    # Keep fob-cli crate version in sync if it has an explicit version
    if grep -q '^version = "' crates/fob-cli/Cargo.toml; then
        sed -i '' "s/^version = \"${CURRENT_VERSION}\"/version = \"${NEW_VERSION}\"/" crates/fob-cli/Cargo.toml
    fi

    echo "🔄 Updating lockfile..."
    cargo check --workspace > /dev/null

    git diff Cargo.toml crates/fob-cli/Cargo.toml Cargo.lock || true

    if gum confirm "Commit and tag v${NEW_VERSION}?"; then
        git add Cargo.toml crates/fob-cli/Cargo.toml Cargo.lock
        git commit -m "chore: bump version to v${NEW_VERSION}"
        git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"
        echo "✨ Tagged v${NEW_VERSION}"

        if gum confirm "Push to origin?"; then
            git push && git push --tags
        else
            echo "💡 Run: git push && git push --tags"
        fi
    else
        echo "⚠️  Changes applied to files but not committed."
    fi

# Build release artifacts for all targets
release: build-native-release build-wasm-release build-napi-release
    @echo "✓ Release build complete!"

# Prepare for release (clean + ci + release)
release-prepare: clean ci release
    @echo "✓ Pre-release checks complete!"

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
    @echo ""
    @echo "WASM tools:"
    @command -v wasm-pack && echo "  wasm-pack: $(wasm-pack --version)" || echo "  wasm-pack: NOT INSTALLED"
    @command -v wasmtime && echo "  wasmtime: $(wasmtime --version)" || echo "  wasmtime: NOT INSTALLED"

# Count lines of code
loc:
    @echo "Lines of code:"
    @find crates -name "*.rs" -not -path "*/target/*" | xargs wc -l | tail -1

# Show dependency tree for a package
deps package:
    @cargo tree --package {{package}}
