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

# Default recipe (show available commands)
default:
    @just --list

# =============================================================================
# Workspace-Level Commands
# =============================================================================

# Check all crates compile
check:
    @echo "Checking all workspace crates..."
    @cargo check --workspace --all-features

# Run all tests
test:
    @echo "Running all tests..."
    @cargo test --workspace

# Run all tests with output
test-verbose:
    @echo "Running all tests (verbose)..."
    @cargo test --workspace -- --nocapture

# Format all Rust code
fmt:
    @echo "Formatting Rust code..."
    @cargo fmt --all

# Check formatting without applying
fmt-check:
    @echo "Checking code formatting..."
    @cargo fmt --all -- --check

# Run clippy on all crates
clippy:
    @echo "Running clippy..."
    @cargo clippy --workspace --all-features -- -D warnings

# Run clippy with pedantic warnings
clippy-pedantic:
    @echo "Running clippy (pedantic)..."
    @cargo clippy --workspace --all-features -- -W clippy::pedantic

# Clean all build artifacts
clean:
    @echo "Cleaning build artifacts..."
    @cargo clean
    @rm -rf crates/fob-wasm/pkg
    @echo "✓ Clean complete"

# Full CI check (format + clippy + test + build + wasm compat)
ci: fmt-check clippy test build-all check-std-fs clippy-wasm check-wasm-wasi wasm-size-report
    @echo "✓ All CI checks passed!"

# Development workflow (format + check + test)
dev: fmt check test
    @echo "✓ Development checks passed!"

# Build everything (all targets)
build-all: build-native build-wasm-all
    @echo "✓ All builds complete!"

# =============================================================================
# Native Builds
# =============================================================================

# Build all native crates
build-native:
    @echo "Building native crates..."
    @cargo build --workspace --exclude fob-wasm

# Build native crates in release mode
build-native-release:
    @echo "Building native crates (release)..."
    @cargo build --workspace --exclude fob-wasm --release

# =============================================================================
# fob-core (Core Library)
# =============================================================================

# Check fob-core compiles
check-core:
    @cargo check --package fob-core --all-features

# Test fob-core
test-core:
    @cargo test --package fob-core --all-features

# Test fob-core with specific feature
test-core-feature feature:
    @cargo test --package fob-core --features {{feature}}

# Build fob-core documentation
docs-core:
    @cargo doc --package fob-core --all-features --no-deps --open

# =============================================================================
# fob-cli (Command-Line Interface)
# =============================================================================

# Build fob-cli in dev mode
build-cli:
    @cargo build --package fob-cli

# Build fob-cli in release mode
build-cli-release:
    @cargo build --package fob-cli --release

# Install fob-cli to cargo bin
install-cli:
    @cargo install --path crates/fob-cli

# Test fob-cli
test-cli:
    @cargo test --package fob-cli

# Run fob-cli with arguments
run-cli *args:
    @cargo run --package fob-cli -- {{args}}

# =============================================================================
# fob-wasm (WASI WASM)
# =============================================================================

# Build WASI WASM (production)
wasm-wasi: wasm-wasi-release

# Build WASI WASM in production mode
wasm-wasi-release:
    @echo "Building WASI WASM (release)..."
    @cd crates/fob-wasm && ./build.sh release

# Build WASI WASM in dev mode
wasm-wasi-dev:
    @echo "Building WASI WASM (dev)..."
    @cd crates/fob-wasm && ./build.sh dev

# Check WASI WASM crate
check-wasm-wasi:
    @cargo check --package fob-wasm --target {{wasm_wasi_target}}

# Show WASI WASM bundle size
wasm-wasi-size:
    @echo "WASI WASM bundle size:"
    @ls -lh crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm 2>/dev/null | awk '{print $5 "\t" $9}' || \
        ls -lh crates/fob-wasm/pkg/debug/fob_bundler_wasm.wasm 2>/dev/null | awk '{print $5 "\t" $9}' || \
        echo "  (No build found - run 'just wasm-wasi' first)"

# Test WASI WASM with wasmtime (requires wasmtime)
test-wasm-wasi:
    @echo "Testing WASI WASM with wasmtime..."
    @wasmtime --version || (echo "Install wasmtime: curl https://wasmtime.dev/install.sh -sSf | bash" && exit 1)
    @test -f crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm || (echo "Build first: just wasm-wasi" && exit 1)
    @wasmtime --wasi threads crates/fob-wasm/pkg/release/fob_bundler_wasm.wasm

# Clean WASI WASM artifacts
clean-wasm-wasi:
    @rm -rf crates/fob-wasm/pkg
    @rm -rf target/{{wasm_wasi_target}}
    @echo "✓ WASI WASM artifacts cleaned"

# =============================================================================
# All WASM Targets
# =============================================================================

# Build all WASM targets (WASI only)
build-wasm-all: wasm-wasi
    @echo "✓ All WASM builds complete!"

# Check all WASM crates compile
check-wasm-all: check-wasm-wasi
    @echo "✓ All WASM crates check passed!"

# Show all WASM bundle sizes
wasm-size-all: wasm-wasi-size

# Report WASM sizes against edge runtime limits
wasm-size-report:
    @echo "📦 WASM Binary Sizes"
    @echo ""
    @if [ -f "packages/fob-edge/wasm/bundler/fob_bundler_wasm_bg.wasm" ]; then \
        SIZE=$$(stat -f%z "packages/fob-edge/wasm/bundler/fob_bundler_wasm_bg.wasm" 2>/dev/null || stat -c%s "packages/fob-edge/wasm/bundler/fob_bundler_wasm_bg.wasm" 2>/dev/null); \
        SIZE_MB=$$(echo "scale=2; $$SIZE / 1024 / 1024" | bc); \
        echo "  Edge (WASI): $${SIZE_MB} MB"; \
        if [ $$(echo "$$SIZE_MB < 3" | bc) -eq 1 ]; then \
            echo "  ✓ Under 3MB (Cloudflare Free)"; \
        elif [ $$(echo "$$SIZE_MB < 10" | bc) -eq 1 ]; then \
            echo "  ✓ Under 10MB (Cloudflare Paid)"; \
        else \
            echo "  ✗ Over 10MB (too large for most edge runtimes)"; \
            exit 1; \
        fi; \
    else \
        echo "  ⚠ WASM file not found (run 'just wasm-wasi' first)"; \
    fi

# Clean all WASM artifacts
clean-wasm-all: clean-wasm-wasi
    @echo "✓ All WASM artifacts cleaned"

# Verify WASM builds work (no std::fs usage)
verify-wasm: check-wasm-all
    @echo "✓ WASM verification passed!"

# Verify WASM compatibility enforcement (clippy + build + checks)
verify-wasm-compat: check-std-fs clippy-wasm check-wasm-wasi
    @echo "✓ WASM compatibility verification passed!"

# Run clippy with WASM-specific checks
clippy-wasm:
    @echo "Running clippy for WASM target..."
    @cargo clippy --package fob-core --target {{wasm_wasi_target}} --all-features -- -D warnings
    @cargo clippy --package fob-wasm --target {{wasm_wasi_target}} -- -D warnings

# Check for accidental std::fs usage (quick check)
check-std-fs:
    @echo "Checking for disallowed std::fs usage..."
    @! grep -r "std::fs" crates/fob-core/src \
        --include="*.rs" \
        --exclude-dir=target \
        | grep -v "test_utils.rs" \
        | grep -v "native_runtime.rs" \
        | grep -v "#\[cfg(test)\]" \
        | grep -v "//.*std::fs" \
        || (echo "❌ Found disallowed std::fs usage! Use Runtime trait instead." && exit 1)
    @echo "✓ No disallowed std::fs usage found"

# =============================================================================
# fob-native (N-API Node.js Bindings)
# =============================================================================

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

# Test N-API bindings
test-napi:
    @cargo test --package fob-native

# Copy native binary to TypeScript package
copy-native:
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
        echo "❌ Native library not found. Run 'just build-napi' or 'just build-napi-release' first"
        exit 1
    fi

    echo "✓ Native binary copied to packages/fob-bundler/index.node"

# Build TypeScript bundler package
build-ts-bundler: copy-native
    @echo "Building TypeScript bundler package..."
    @cd packages/fob-bundler && pnpm install
    @cd packages/fob-bundler && pnpm build
    @echo "✓ TypeScript bundler built"

# Full bundler build (development)
build-bundler: build-napi build-ts-bundler
    @echo "✓ Bundler development build complete!"

# Full bundler build (release)
build-bundler-release: build-napi-release build-ts-bundler
    @echo "✓ Bundler release build complete!"

# =============================================================================
# fob-config (Configuration)
# =============================================================================

# Test fob-config
test-config:
    @cargo test --package fob-config --all-features

# Test fob-config with eval feature
test-config-eval:
    @cargo test --package fob-config --features eval

# =============================================================================
# fob-docs (Documentation Generator)
# =============================================================================

# Test fob-docs
test-docs:
    @cargo test --package fob-docs --all-features

# Test fob-docs with LLM feature
test-docs-llm:
    @cargo test --package fob-docs --features llm

# =============================================================================
# Gumbo (Web Framework)
# =============================================================================

# Build gumbo-cli
build-gumbo:
    @cargo build --package gumbo-cli

# Build gumbo-cli (release)
build-gumbo-release:
    @cargo build --package gumbo-cli --release

# Install gumbo CLI
install-gumbo:
    @cargo install --path crates/gumbo-cli

# Test gumbo-core
test-gumbo:
    @cargo test --package gumbo-core
    @cargo test --package gumbo-cli

# =============================================================================
# fob-plugin-css (CSS Plugin)
# =============================================================================

# Test CSS plugin
test-plugin-css:
    @cargo test --package fob-plugin-css --all-features

# Check CSS plugin compiles
check-plugin-css:
    @cargo check --package fob-plugin-css --all-features

# =============================================================================
# fob-plugin-mdx (MDX Plugin)
# =============================================================================

# Test MDX plugin
test-plugin-mdx:
    @cargo test --package fob-plugin-mdx --all-features

# Check MDX plugin compiles
check-plugin-mdx:
    @cargo check --package fob-plugin-mdx --all-features

# =============================================================================
# fob-plugin-tailwind (Tailwind Plugin)
# =============================================================================

# Test Tailwind plugin
test-plugin-tailwind:
    @cargo test --package fob-plugin-tailwind --all-features

# Check Tailwind plugin compiles
check-plugin-tailwind:
    @cargo check --package fob-plugin-tailwind --all-features

# =============================================================================
# fob-gen (Code Generation)
# =============================================================================

# Test fob-gen
test-gen:
    @cargo test --package fob-gen --all-features

# Check fob-gen compiles
check-gen:
    @cargo check --package fob-gen --all-features

# Build fob-gen documentation
docs-gen:
    @cargo doc --package fob-gen --all-features --no-deps --open

# =============================================================================
# fob-browser-test (Browser Testing)
# =============================================================================

# Test browser-test
test-browser:
    @cargo test --package fob-browser-test

# Check browser-test compiles
check-browser:
    @cargo check --package fob-browser-test

# =============================================================================
# Plugin Development Workflows
# =============================================================================

# Test all plugins together
test-plugins: test-plugin-css test-plugin-mdx test-plugin-tailwind
    @echo "✓ All plugin tests passed!"

# Check all plugins compile
check-plugins: check-plugin-css check-plugin-mdx check-plugin-tailwind
    @echo "✓ All plugins check passed!"

# Full plugin development workflow
dev-plugins: check-plugins test-plugins
    @echo "✓ Plugin development checks complete!"

# =============================================================================
# Verification & Quality
# =============================================================================

# Verify all builds work correctly
verify-all: verify-native verify-wasm verify-tests
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

# Build release artifacts for all targets
release: build-native-release wasm-wasi build-napi-release
    @echo "✓ Release build complete!"

# Prepare for release (checks + tests + builds)
pre-release: clean ci release
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

# List all available examples (default)
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
# Installation Targets
# =============================================================================

# Install Rust targets for WASM builds
install-targets:
    @echo "Installing Rust WASM targets..."
    @rustup target add {{wasm_wasi_target}}
    @echo "✓ WASM targets installed"

# Install build tools (wasm-pack, cargo-watch, etc.)
install-tools:
    @echo "Installing build tools..."
    @command -v wasm-pack || cargo install wasm-pack
    @command -v cargo-watch || cargo install cargo-watch
    @command -v cargo-audit || cargo install cargo-audit
    @command -v wasmtime || echo "Install wasmtime: curl https://wasmtime.dev/install.sh -sSf | bash"
    @echo "✓ Build tools installed"

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

# Run tests for a specific package
test-package package:
    @cargo test --package {{package}}
