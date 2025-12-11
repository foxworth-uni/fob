#!/usr/bin/env php
<?php
/**
 * Test inline content feature with Fob bundler (PHP)
 *
 * Demonstrates bundling inline JavaScript/TypeScript code without file I/O.
 */

// Load the native extension
if (!extension_loaded('fob-php')) {
    echo "❌ Error: fob extension not found!\n";
    echo "\nTo build the extension:\n";
    echo "  cd ../../..\n";
    echo "  cargo build --package fob-php\n";
    echo "\nOr for release build:\n";
    echo "  cargo build --release --package fob-php\n";
    exit(1);
}

function main() {
    echo "🚀 Testing inline content with Fob (PHP)...\n\n";

    // Create a temporary directory for output (in project dir for security)
    $script_dir = __DIR__;
    $temp_dir = "{$script_dir}/.tmp-inline-test";
    if (!is_dir($temp_dir)) {
        mkdir($temp_dir, 0777, true);
    }
    echo "📁 Output directory: {$temp_dir}\n\n";

    try {
        // Initialize logging
        init_logging('info');

        // Test 1: Single inline content entry
        echo "Test 1: Single inline content entry\n";
        $bundler1 = new Fob([
            'entries' => [[
                'content' => "console.log('Hello from PHP inline content!');",
                'name' => 'main.js',
            ]],
            'output_dir' => "{$temp_dir}/test1",
            'format' => 'esm',
        ]);
        $result1 = $bundler1->bundle();
        $chunk1 = $result1['chunks'][0];
        echo "✅ Generated: {$chunk1['file_name']} ({$chunk1['size']} bytes)\n\n";

        // Test 2: Multiple inline content entries
        echo "Test 2: Multiple inline content entries\n";
        $bundler2 = new Fob([
            'entries' => [
                [
                    'content' => "console.log('Entry 1: Hello from inline!');",
                    'name' => 'entry1.js',
                ],
                [
                    'content' => "console.log('Entry 2: Another inline file!');",
                    'name' => 'entry2.js',
                ]
            ],
            'output_dir' => "{$temp_dir}/test2",
            'format' => 'esm',
        ]);
        $result2 = $bundler2->bundle();
        echo "✅ Chunks generated:\n";
        foreach ($result2['chunks'] as $chunk) {
            echo "  - {$chunk['file_name']} ({$chunk['size']} bytes)\n";
        }
        echo "\n";

        // Test 3: Mixed inline and file entries
        echo "Test 3: Mixed inline content and file path\n";
        $script_dir = __DIR__;
        $bundler3 = new Fob([
            'entries' => [
                "{$script_dir}/src/index.js",  // File path
                [
                    'content' => "console.log('Plus inline content!');",
                    'name' => 'inline.js',
                ]
            ],
            'output_dir' => "{$temp_dir}/test3",
            'format' => 'esm',
        ]);
        $result3 = $bundler3->bundle();
        echo "✅ Chunks generated:\n";
        foreach ($result3['chunks'] as $chunk) {
            echo "  - {$chunk['file_name']} ({$chunk['size']} bytes)\n";
        }
        echo "\n";

        // Test 4: TypeScript inline content
        echo "Test 4: TypeScript inline content\n";
        $bundler4 = new Fob([
            'entries' => [[
                'content' => "const message: string = 'TypeScript works!'; console.log(message);",
                'name' => 'typed.ts',
                'loader' => 'ts',
            ]],
            'output_dir' => "{$temp_dir}/test4",
            'format' => 'esm',
        ]);
        $result4 = $bundler4->bundle();
        $chunk4 = $result4['chunks'][0];
        echo "✅ Generated: {$chunk4['file_name']} ({$chunk4['size']} bytes)\n\n";

        echo "✅ All tests passed!\n\n";
        echo "📊 Summary:\n";
        echo "  Test 1: {$result1['stats']['total_modules']} modules, {$result1['stats']['total_size']} bytes\n";
        echo "  Test 2: {$result2['stats']['total_modules']} modules, {$result2['stats']['total_size']} bytes\n";
        echo "  Test 3: {$result3['stats']['total_modules']} modules, {$result3['stats']['total_size']} bytes\n";
        echo "  Test 4: {$result4['stats']['total_modules']} modules, {$result4['stats']['total_size']} bytes\n";

    } catch (Exception $e) {
        echo "❌ Test failed: {$e->getMessage()}\n";
        exit(1);
    } finally {
        // Clean up temp directory
        echo "\n🧹 Cleaning up {$temp_dir}\n";
        if (is_dir($temp_dir)) {
            $it = new RecursiveDirectoryIterator($temp_dir, RecursiveDirectoryIterator::SKIP_DOTS);
            $files = new RecursiveIteratorIterator($it, RecursiveIteratorIterator::CHILD_FIRST);
            foreach($files as $file) {
                if ($file->isDir()){
                    rmdir($file->getRealPath());
                } else {
                    unlink($file->getRealPath());
                }
            }
            rmdir($temp_dir);
        }
    }
}

main();
