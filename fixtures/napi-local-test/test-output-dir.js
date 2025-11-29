import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, rmSync, readdirSync, existsSync } from 'fs';
import { Fob, OutputFormat } from '@fox-uni/fob';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('\n🧪 Testing output_dir with subdirectory fix\n');

// Test: output_dir with subdirectory (the bug we're fixing)
console.log('📦 Test: output_dir with subdirectory (dist/test-eprintln)');
try {
  const outputDir = join(__dirname, 'dist', 'test-eprintln');

  // Clean entire dist directory to avoid false positives from stale files
  rmSync(join(__dirname, 'dist'), { recursive: true, force: true });

  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: outputDir, // Correct property name (camelCase for NAPI)
    format: OutputFormat.Esm,
    cwd: __dirname,
  };

  console.log(`   Config outputDir: ${config.outputDir}`);

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Bundled ${result.moduleCount} module(s)`);

  // Verify files were written to the correct subdirectory
  const expectedDir = join(__dirname, 'dist', 'test-eprintln');
  if (!existsSync(expectedDir)) {
    console.error(`❌ Output directory does not exist: ${expectedDir}`);
    process.exit(1);
  }

  const files = readdirSync(expectedDir);
  console.log(`   Files in ${expectedDir}:`, files);

  if (files.length === 0) {
    console.error(`❌ No files written to output directory!`);
    process.exit(1);
  }

  // Check if files are in the correct location (not in dist/ directly)
  const distFiles = readdirSync(join(__dirname, 'dist')).filter(
    (f) => f !== 'test-eprintln' && !f.startsWith('.')
  );

  if (distFiles.some((f) => f === 'index.js' || f.endsWith('.js'))) {
    console.error(`❌ Files were written to dist/ instead of dist/test-eprintln/`);
    console.error(`   Files in dist/:`, distFiles);
    process.exit(1);
  }

  console.log(`✅ Files correctly written to: ${expectedDir}\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  console.error(err.stack);
  process.exit(1);
}

console.log('✨ output_dir subdirectory test passed!\n');
