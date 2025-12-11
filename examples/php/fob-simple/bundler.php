#!/usr/bin/env php
<?php
/**
 * Simple Fob Bundler Example
 *
 * This demonstrates the most basic way to use Fob from PHP.
 * Perfect for getting started with JavaScript bundling in PHP!
 */

// Load the native extension
// In production, this would be loaded via php.ini: extension=fob.so
// For development, check if extension is loaded
if (!extension_loaded('fob-php')) {
    echo "❌ Error: fob extension not found!\n";
    echo "\nTo build the extension:\n";
    echo "  cd ../../..\n";
    echo "  cargo build --package fob-php\n";
    echo "\nOr for release build:\n";
    echo "  cargo build --release --package fob-php\n";
    echo "\nThen install it:\n";
    echo "  cargo php install --release\n";
    echo "\nOr manually:\n";
    echo "  1. Find extension dir: php -i | grep extension_dir\n";
    echo "  2. Copy: cp target/release/libfob.so <extension_dir>/fob.so\n";
    echo "  3. Add to php.ini: extension=fob.so\n";
    exit(1);
}

function main() {
    echo "🚀 Building with Fob...\n\n";

    try {
        // Initialize logging (optional, defaults to info level)
        init_logging('info');

        // Method 1: Using the simple bundle_single helper
        // This is the easiest way to bundle a single file
        $script_dir = __DIR__;
        $entry_path = $script_dir . '/src/index.js';
        $output_dir = $script_dir . '/dist';

        // Change to script directory so relative imports work correctly
        $original_cwd = getcwd();
        chdir($script_dir);

        try {
            $result = bundle_single(
                $entry_path,
                $output_dir,
                'esm'
            );
        } finally {
            chdir($original_cwd);
        }

        // Method 2: Using the Fob class for more control
        // Uncomment to try this instead:
        // $bundler = new Fob([
        //     'entries' => [$entry_path],
        //     'output_dir' => $output_dir,
        //     'format' => 'esm',
        //     'sourcemap' => 'external',
        // ]);
        // $result = $bundler->bundle();

        // Method 3: Using preset methods
        // Uncomment to try this instead:
        // $result = bundle_entry(
        //     $entry_path,
        //     ['out_dir' => $output_dir, 'format' => 'esm']
        // );

        // Display results
        echo "✅ Build complete!\n\n";

        echo "📦 Chunks generated:\n";
        foreach ($result['chunks'] as $chunk) {
            echo "  - {$chunk['file_name']} ({$chunk['size']} bytes)\n";
        }

        echo "\n📊 Build stats:\n";
        $stats = $result['stats'];
        echo "  Modules: {$stats['total_modules']}\n";
        echo "  Chunks: {$stats['total_chunks']}\n";
        echo "  Total size: {$stats['total_size']} bytes\n";
        echo "  Duration: {$stats['duration_ms']}ms\n";
        echo "  Cache hit rate: " . number_format($stats['cache_hit_rate'] * 100, 1) . "%\n";

        if (!empty($result['assets'])) {
            echo "\n📁 Assets:\n";
            foreach ($result['assets'] as $asset) {
                echo "  - {$asset['relative_path']} ({$asset['size']} bytes)\n";
            }
        }

        echo "\n💡 Output written to: dist/\n";
        echo "\n🚀 Try it:\n";
        echo "   node dist/index.js\n";

    } catch (Exception $e) {
        echo "❌ Build failed: {$e->getMessage()}\n";
        exit(1);
    }
}

main();
