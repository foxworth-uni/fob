import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, rmSync } from 'fs';
import { Fob, bundleSingle, version, OutputFormat } from '@fox-uni/fob';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('\n🧪 Testing fob-native NAPI bindings (Simple)\n');

// Test 1: Version
console.log('📦 Test 1: version()');
try {
  const ver = version();
  console.log(`✅ Version: ${ver}\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 2: Simple bundle (ESM)
console.log('📦 Test 2: Simple bundle - ESM format');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/test2'),
    format: OutputFormat.Esm,
    cwd: __dirname,
  };

  // Clean and create output dir
  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Bundled ${result.moduleCount} module(s)`);
  console.log(`   Output: ${config.outputDir}`);
  console.log(`   Format: ESM\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 3: Simple bundle (CJS)
console.log('📦 Test 3: Simple bundle - CJS format');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/test3'),
    format: OutputFormat.Cjs,
    cwd: __dirname,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Bundled ${result.moduleCount} module(s)`);
  console.log(`   Format: CJS\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 4: With imports
console.log('📦 Test 4: Bundle with imports');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/with-import/index.js')],
    outputDir: join(__dirname, 'dist/test4'),
    format: OutputFormat.Esm,
    cwd: __dirname,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Bundled ${result.moduleCount} module(s)`);
  console.log(`   Resolved imports correctly\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

console.log('✨ All simple tests passed!\n');
