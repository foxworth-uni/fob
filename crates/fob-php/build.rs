//! Build script for fob-php
//!
//! PHP extensions link against symbols provided by the PHP runtime at load time.
//! This build script configures the linker to allow undefined symbols.

fn main() {
    // On macOS, tell the linker to allow undefined symbols (resolved at runtime)
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-cdylib-link-arg=-undefined");
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }

    // On Linux, mark as allowing undefined symbols
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-cdylib-link-arg=-Wl,--allow-shlib-undefined");
    }
}
