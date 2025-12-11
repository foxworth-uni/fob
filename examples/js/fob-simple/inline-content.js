import pkg from '@fox-uni/fob';
const { Fob } = pkg;
import { mkdirSync, rmSync } from 'fs';
import { join } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function main() {
  console.log('🚀 Testing inline content with Fob (Node.js)...\n');

  // Create a temporary directory for output (in project dir for security)
  const tempDir = join(__dirname, '.tmp-inline-test');
  mkdirSync(tempDir, { recursive: true });
  console.log(`📁 Output directory: ${tempDir}\n`);

  try {
    // Test 1: Single inline content entry
    console.log('Test 1: Single inline content entry');
    const bundler1 = new Fob({
      entries: [
        {
          content: "console.log('Hello from Node.js inline content!');",
          name: 'main.js',
        },
      ],
      outputDir: join(tempDir, 'test1'),
      format: 'Esm',
    });
    const result1 = await bundler1.bundle();
    console.log(`✅ Generated: ${result1.chunks[0].fileName} (${result1.chunks[0].size} bytes)\n`);

    // Test 2: Multiple inline content entries
    console.log('Test 2: Multiple inline content entries');
    const bundler2 = new Fob({
      entries: [
        {
          content: "console.log('Entry 1: Hello from inline!');",
          name: 'entry1.js',
        },
        {
          content: "console.log('Entry 2: Another inline file!');",
          name: 'entry2.js',
        },
      ],
      outputDir: join(tempDir, 'test2'),
      format: 'Esm',
    });
    const result2 = await bundler2.bundle();
    console.log('✅ Chunks generated:');
    for (const chunk of result2.chunks) {
      console.log(`  - ${chunk.fileName} (${chunk.size} bytes)`);
    }
    console.log();

    // Test 3: Mixed inline and file entries
    console.log('Test 3: Mixed inline content and file path');
    const bundler3 = new Fob({
      entries: [
        'src/index.js', // File path
        {
          content: "console.log('Plus inline content!');",
          name: 'inline.js',
        },
      ],
      outputDir: join(tempDir, 'test3'),
      format: 'Esm',
    });
    const result3 = await bundler3.bundle();
    console.log('✅ Chunks generated:');
    for (const chunk of result3.chunks) {
      console.log(`  - ${chunk.fileName} (${chunk.size} bytes)`);
    }
    console.log();

    // Test 4: TypeScript inline content
    console.log('Test 4: TypeScript inline content');
    const bundler4 = new Fob({
      entries: [
        {
          content: "const message: string = 'TypeScript works!'; console.log(message);",
          name: 'typed.ts',
          loader: 'ts',
        },
      ],
      outputDir: join(tempDir, 'test4'),
      format: 'Esm',
    });
    const result4 = await bundler4.bundle();
    console.log(`✅ Generated: ${result4.chunks[0].fileName} (${result4.chunks[0].size} bytes)\n`);

    console.log('✅ All tests passed!\n');
    console.log('📊 Summary:');
    console.log(
      `  Test 1: ${result1.stats.totalModules} modules, ${result1.stats.totalSize} bytes`
    );
    console.log(
      `  Test 2: ${result2.stats.totalModules} modules, ${result2.stats.totalSize} bytes`
    );
    console.log(
      `  Test 3: ${result3.stats.totalModules} modules, ${result3.stats.totalSize} bytes`
    );
    console.log(
      `  Test 4: ${result4.stats.totalModules} modules, ${result4.stats.totalSize} bytes`
    );
  } catch (error) {
    console.error('❌ Test failed:', error.message);
    process.exit(1);
  } finally {
    // Clean up temp directory
    console.log(`\n🧹 Cleaning up ${tempDir}`);
    rmSync(tempDir, { recursive: true, force: true });
  }
}

main();
