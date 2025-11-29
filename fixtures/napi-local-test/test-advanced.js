import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, rmSync, existsSync, readFileSync } from 'fs';
import { Fob, OutputFormat, SourceMapMode } from '@fox-uni/fob';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('\n🧪 Testing fob-native NAPI bindings (Advanced)\n');

// Test 1: Multiple entries
console.log('📦 Test 1: Multiple entry points');
try {
  const config = {
    entries: [
      join(__dirname, 'fixtures/multi-entry/a.js'),
      join(__dirname, 'fixtures/multi-entry/b.js'),
    ],
    outputDir: join(__dirname, 'dist/advanced-1'),
    bundle: true,
    format: OutputFormat.Esm,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Bundled ${result.moduleCount} module(s) from ${config.entries.length} entries\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 2: IIFE format
console.log('📦 Test 2: IIFE format (browser bundle)');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/advanced-2'),
    bundle: true,
    format: OutputFormat.Iife,
    cwd: __dirname,
    platform: 'browser',
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Created IIFE bundle for browser\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 3: Inline sourcemaps
console.log('📦 Test 3: Inline sourcemaps');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/advanced-3'),
    bundle: true,
    format: OutputFormat.Esm,
    cwd: __dirname,
    sourcemap: SourceMapMode.Inline,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  // Check that bundle contains inline sourcemap
  const outputFile = join(config.outputDir, result.chunks[0].fileName);
  const content = readFileSync(outputFile, 'utf-8');
  const hasInlineMap = content.includes('//# sourceMappingURL=data:');

  if (!hasInlineMap) {
    throw new Error('Expected inline sourcemap not found');
  }

  console.log(`✅ Generated inline sourcemap\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 4: External sourcemaps
console.log('📦 Test 4: External sourcemaps');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/advanced-4'),
    bundle: true,
    format: OutputFormat.Esm,
    cwd: __dirname,
    sourcemap: SourceMapMode.External,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  // Check that .map file exists
  const mapFile = join(config.outputDir, result.chunks[0].fileName + '.map');
  if (!existsSync(mapFile)) {
    throw new Error('Expected external .map file not found');
  }

  console.log(`✅ Generated external sourcemap file\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 5: No sourcemap
console.log('📦 Test 5: Disabled sourcemaps');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/advanced-5'),
    bundle: true,
    format: OutputFormat.Esm,
    cwd: __dirname,
    sourcemap: SourceMapMode.Disabled,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  // Check that no sourcemap exists
  const outputFile = join(config.outputDir, result.chunks[0].fileName);
  const content = readFileSync(outputFile, 'utf-8');
  const hasAnyMap = content.includes('sourceMappingURL') || existsSync(outputFile + '.map');

  if (hasAnyMap) {
    throw new Error('Unexpected sourcemap found when disabled');
  }

  console.log(`✅ No sourcemap generated (as expected)\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

// Test 6: Minification
console.log('📦 Test 6: Minification');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/advanced-6'),
    bundle: true,
    format: OutputFormat.Esm,
    cwd: __dirname,
    minify: true,
  };

  rmSync(config.outputDir, { recursive: true, force: true });
  mkdirSync(config.outputDir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log(`✅ Minified bundle created\n`);
} catch (err) {
  console.error('❌ Failed:', err.message);
  process.exit(1);
}

console.log('✨ All advanced tests passed!\n');
