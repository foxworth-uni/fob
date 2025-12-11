# Fob PHP Example

Simple example demonstrating how to use Fob bundler from PHP.

## Prerequisites

- PHP 8.1-8.4 with development headers (PHP 8.5 not yet supported - see Known Issues)
- Rust 1.85+
- Clang 5.0+ (for bindgen)

## Known Issues

**PHP 8.5 Compatibility:** PHP 8.5 is not yet fully supported due to a bindgen compatibility issue. Please use PHP 8.4 or earlier until this is resolved.

## Building the Extension

### 1. Install PHP Development Headers

**macOS (Homebrew):**

```bash
brew install php
```

**Ubuntu/Debian:**

```bash
sudo apt-get install php-dev php-cli
```

**Fedora/RHEL:**

```bash
sudo dnf install php-devel php-cli
```

### 2. Build the Extension

```bash
# Debug build
cargo build --package fob-php

# Release build (recommended)
cargo build --release --package fob-php
```

### 3. Install the Extension

**Option A: Using cargo-php (recommended)**

```bash
# Install cargo-php if you haven't already
cargo install cargo-php

# Install the extension
cargo php install --release
```

**Option B: Manual Installation**

1. Find your PHP extension directory:

   ```bash
   php -i | grep extension_dir
   ```

2. Copy the built extension:

   ```bash
   cp target/release/libfob.so $(php-config --extension-dir)/fob.so
   ```

3. Add to php.ini:
   ```ini
   extension=fob.so
   ```

**Option C: Load Dynamically (for testing)**

The example scripts will try to load the extension from the build directory automatically.

## Running the Examples

### Simple Example

```bash
php bundler.php
```

This will:

- Bundle `src/index.js` and its dependencies
- Output to `dist/` directory
- Display build statistics

### Advanced Example

```bash
php advanced_example.php
```

This demonstrates:

- Library builds with external dependencies
- App builds with code splitting
- Component library builds
- Custom configuration options

## API Reference

### Functions

- `fob_init_logging(?string $level): void` - Initialize logging
- `fob_init_logging_from_env(): void` - Initialize logging from RUST_LOG
- `fob_bundle_single(string $entry, string $outputDir, ?string $format): array` - Quick bundle helper
- `fob_version(): string` - Get bundler version

### Preset Functions

- `fob_bundle_entry(string $entry, ?array $options): array` - Bundle single entry
- `fob_library(string $entry, ?array $options): array` - Build library (externalize deps)
- `fob_app(array $entries, ?array $options): array` - Build app (code splitting)
- `fob_components(array $entries, ?array $options): array` - Build components (isolated)

### Fob Class

```php
$bundler = new Fob([
    'entries' => ['src/index.js'],
    'output_dir' => 'dist',
    'format' => 'esm',           // 'esm', 'cjs', or 'iife'
    'sourcemap' => 'external',   // true, false, 'inline', 'hidden', 'external'
    'platform' => 'browser',     // 'browser' or 'node'
    'minify' => false,
    'cwd' => __DIR__,
    'external' => ['react'],     // Packages to externalize
    'entry_mode' => 'shared',    // 'shared' or 'isolated'
    'code_splitting' => [
        'min_size' => 20000,
        'min_imports' => 2,
    ],
]);

$result = $bundler->bundle();
```

### Result Structure

```php
[
    'chunks' => [
        [
            'id' => 'index.js',
            'kind' => 'entry',
            'file_name' => 'index.js',
            'code' => '...',
            'source_map' => '...',
            'modules' => [...],
            'imports' => [...],
            'dynamic_imports' => [...],
            'size' => 1234,
        ],
    ],
    'manifest' => [
        'entries' => [...],
        'chunks' => [...],
        'version' => '...',
    ],
    'stats' => [
        'total_modules' => 5,
        'total_chunks' => 1,
        'total_size' => 1234,
        'duration_ms' => 100,
        'cache_hit_rate' => 0.5,
    ],
    'assets' => [...],
    'module_count' => 5,
]
```

## Troubleshooting

### Extension Not Found

If you see "fob extension not found":

1. Make sure you've built the extension: `cargo build --release --package fob-php`
2. Check that the extension is in the PHP extension directory
3. Verify php.ini includes: `extension=fob.so`
4. Restart PHP-FPM or your web server if applicable

### Build Errors

If cargo build fails:

- Ensure PHP development headers are installed
- Check that PHP is in your PATH: `which php`
- Verify PHP version: `php -v` (needs 8.1+)

### Runtime Errors

- Check PHP error logs: `php -i | grep error_log`
- Enable error reporting: `ini_set('display_errors', 1);`
- Verify entry files exist and are readable

## Next Steps

- Check out the [main Fob documentation](../../../README.md)
- Explore [advanced examples](../../../examples/)
- See [Python examples](../python/fob-simple/) for comparison
- See [Ruby examples](../ruby/fob-simple/) for comparison
