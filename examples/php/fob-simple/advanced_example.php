#!/usr/bin/env php
<?php
/**
 * Advanced Fob Bundler Example
 *
 * Demonstrates advanced features like:
 * - Library builds with external dependencies
 * - App builds with code splitting
 * - Component library builds
 * - Custom configuration options
 */

// Load the native extension
if (!extension_loaded('fob')) {
    echo "❌ Error: fob extension not found!\n";
    echo "Build and install the extension first:\n";
    echo "  cargo build --release --package fob-php\n";
    echo "  cargo php install --release\n";
    exit(1);
}

function example_library() {
    echo "📚 Example 1: Library Build (externalize dependencies)\n";
    echo str_repeat('-', 60) . "\n";
    
    try {
        // Build a library - dependencies are externalized
        $result = fob_library(
            'src/index.js',
            [
                'out_dir' => 'dist/library',
                'format' => 'esm',
                'external' => ['react', 'react-dom'], // Externalize these packages
                'sourcemap' => 'external',
            ]
        );
        
        echo "✅ Library build complete!\n";
        echo "   Chunks: " . count($result['chunks']) . "\n";
        echo "   Modules: {$result['stats']['total_modules']}\n\n";
        
    } catch (Exception $e) {
        echo "❌ Failed: {$e->getMessage()}\n\n";
    }
}

function example_app() {
    echo "🚀 Example 2: App Build (with code splitting)\n";
    echo str_repeat('-', 60) . "\n";
    
    try {
        // Build an app with code splitting
        $result = fob_app(
            ['src/index.js', 'src/utils.js'], // Multiple entries
            [
                'out_dir' => 'dist/app',
                'format' => 'esm',
                'code_splitting' => [
                    'min_size' => 20000,
                    'min_imports' => 2,
                ],
                'minify' => true,
            ]
        );
        
        echo "✅ App build complete!\n";
        echo "   Chunks: " . count($result['chunks']) . "\n";
        echo "   Total size: {$result['stats']['total_size']} bytes\n\n";
        
    } catch (Exception $e) {
        echo "❌ Failed: {$e->getMessage()}\n\n";
    }
}

function example_components() {
    echo "🧩 Example 3: Component Library Build\n";
    echo str_repeat('-', 60) . "\n";
    
    try {
        // Build a component library - each entry is isolated
        $result = fob_components(
            ['src/index.js', 'src/utils.js'],
            [
                'out_dir' => 'dist/components',
                'format' => 'esm',
                'external_from_manifest' => true, // Externalize from package.json
            ]
        );
        
        echo "✅ Component library build complete!\n";
        echo "   Chunks: " . count($result['chunks']) . "\n";
        echo "   Module count: {$result['module_count']}\n\n";
        
    } catch (Exception $e) {
        echo "❌ Failed: {$e->getMessage()}\n\n";
    }
}

function example_custom_config() {
    echo "⚙️  Example 4: Custom Configuration\n";
    echo str_repeat('-', 60) . "\n";
    
    try {
        // Use the Fob class for full control
        $bundler = new Fob([
            'entries' => ['src/index.js'],
            'output_dir' => 'dist/custom',
            'format' => 'esm',
            'sourcemap' => 'inline',
            'platform' => 'browser',
            'minify' => false,
            'cwd' => __DIR__,
        ]);
        
        $result = $bundler->bundle();
        
        echo "✅ Custom build complete!\n";
        echo "   Format: esm\n";
        echo "   Sourcemap: inline\n";
        echo "   Platform: browser\n";
        echo "   Duration: {$result['stats']['duration_ms']}ms\n\n";
        
    } catch (Exception $e) {
        echo "❌ Failed: {$e->getMessage()}\n\n";
    }
}

function main() {
    echo "🎯 Advanced Fob Examples\n\n";
    
    // Initialize logging
    fob_init_logging('info');
    
    // Run examples
    example_library();
    example_app();
    example_components();
    example_custom_config();
    
    echo "✨ All examples complete!\n";
}

main();
