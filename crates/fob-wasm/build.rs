/// Build script for fob-wasm
///
/// This script validates that the crate is being built for the correct target.
/// WASM Component Model crates using wit-bindgen MUST be built for wasm32 targets.

fn main() {
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    // Check if building for wasm32
    if !target.starts_with("wasm32-") {
        eprintln!("\n╔════════════════════════════════════════════════════════════════╗");
        eprintln!("║  ❌ ERROR: fob-wasm must be built for wasm32 targets          ║");
        eprintln!("╚════════════════════════════════════════════════════════════════╝");
        eprintln!();
        eprintln!("  Current target: {}", target);
        eprintln!();
        eprintln!("  This crate uses WASM Component Model (wit-bindgen) which only");
        eprintln!("  works on wasm32 targets. Building for native targets (like");
        eprintln!("  arm64, x86_64) will fail with linker errors.");
        eprintln!();
        eprintln!("╭─ ✅ Recommended Solution ──────────────────────────────────────╮");
        eprintln!("│                                                                │");
        eprintln!("│  Use the build script (handles everything):                   │");
        eprintln!("│                                                                │");
        eprintln!("│    cd crates/fob-wasm                                          │");
        eprintln!("│    ./build.sh dev      # or ./build.sh release                │");
        eprintln!("│                                                                │");
        eprintln!("╰────────────────────────────────────────────────────────────────╯");
        eprintln!();
        eprintln!("╭─ Alternative: Specify Target Manually ─────────────────────────╮");
        eprintln!("│                                                                │");
        eprintln!("│  If you need direct cargo control:                            │");
        eprintln!("│                                                                │");
        eprintln!("│    # First, add the target:                                   │");
        eprintln!("│    rustup target add wasm32-wasip1                            │");
        eprintln!("│                                                                │");
        eprintln!("│    # Then build:                                              │");
        eprintln!("│    cargo build --target wasm32-wasip1 --package fob-wasm     │");
        eprintln!("│                                                                │");
        eprintln!("╰────────────────────────────────────────────────────────────────╯");
        eprintln!();
        eprintln!("  For more information, see: crates/fob-wasm/README.md");
        eprintln!();

        std::process::exit(1);
    }

    // Target is wasm32, proceed with build
    println!("cargo:rerun-if-changed=wit/bundler.wit");
    println!("cargo:rerun-if-changed=build.rs");
}
