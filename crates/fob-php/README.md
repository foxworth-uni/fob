# fob-php

PHP bindings for Fob bundler core using ext-php-rs.

## Requirements

- PHP 8.1+ with development headers
- Rust 1.85+
- Clang 5.0+ (for bindgen)

## Known Issues

### PHP 8.5 Compatibility

**Current Limitation:** PHP 8.5 is not yet fully supported due to a bindgen compatibility issue. The `bindgen` crate (v0.72.1) used by `ext-php-rs` doesn't support PHP 8.5's calling convention (error: "Cannot turn unknown calling convention to tokens: 20").

**Workaround:** Use PHP 8.4 or earlier until bindgen/ext-php-rs is updated to support PHP 8.5.

**Status:** This is a known issue and will be resolved when:

- bindgen is updated to support PHP 8.5's calling convention, or
- ext-php-rs is updated to use a newer bindgen version

## Building

```bash
# Debug build
cargo build --package fob-php

# Release build
cargo build --release --package fob-php
```

## Installation

After building, install the extension:

```bash
# Using cargo-php (recommended)
cargo install cargo-php
cargo php install --release

# Or manually:
cp target/release/libfob.so $(php-config --extension-dir)/fob.so
# Then add to php.ini: extension=fob.so
```

## Usage

See `examples/php/fob-simple/` for usage examples.

## API

- Functions: `fob_init_logging()`, `fob_bundle_single()`, `fob_version()`, etc.
- Preset functions: `fob_bundle_entry()`, `fob_library()`, `fob_app()`, `fob_components()`
- Fob class: `new Fob($config)` with `bundle()` method

See the examples directory for detailed usage.
