import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, rmSync } from 'fs';
import { Fob, bundleSingle, OutputFormat } from 'fob-native-build';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('\n🧪 Testing fob-native NAPI bindings (Error Handling)\n');

// Test 1: Non-existent entry file
console.log('📦 Test 1: Non-existent entry file');
try {
  const entryPath = join(__dirname, 'fixtures/does-not-exist.js');
  const outputDir = join(__dirname, 'dist/error-1');

  mkdirSync(outputDir, { recursive: true });

  await bundleSingle(entryPath, outputDir);
  console.error('❌ Should have thrown an error!');
  process.exit(1);
} catch (err) {
  // Try to parse as JSON error
  try {
    const errorData = JSON.parse(err.message);
    console.log(`✅ Caught expected error:`);
    console.log(`   Kind: ${errorData.kind}`);
    console.log(`   Message: ${errorData.message}\n`);
  } catch {
    // Fallback for non-JSON errors
    console.log(`✅ Caught expected error: ${err.message}\n`);
  }
}

// Test 2: Syntax error in source file
console.log('📦 Test 2: Syntax error in source file');
try {
  const entryPath = join(__dirname, 'fixtures/error-case/syntax-error.js');
  const outputDir = join(__dirname, 'dist/error-2');

  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(outputDir, { recursive: true });

  await bundleSingle(entryPath, outputDir);
  console.error('❌ Should have thrown a parse error!');
  process.exit(1);
} catch (err) {
  try {
    const errorData = JSON.parse(err.message);
    console.log(`✅ Caught expected parse error:`);
    console.log(`   Kind: ${errorData.kind}`);
    console.log(`   File: ${errorData.file || 'N/A'}`);
    console.log(`   Line: ${errorData.line || 'N/A'}`);
    console.log(`   Message: ${errorData.message}\n`);
  } catch {
    console.log(`✅ Caught expected parse error: ${err.message}\n`);
  }
}

// Test 3: Invalid configuration
console.log('📦 Test 3: Invalid configuration (empty entry)');
try {
  const config = {
    entries: [],  // Invalid: empty array
    outputDir: join(__dirname, 'dist/error-3'),
    bundle: true,
    format: OutputFormat.Esm,
  };

  const bundler = new Fob(config);
  await bundler.bundle();
  console.error('❌ Should have thrown a config error!');
  process.exit(1);
} catch (err) {
  try {
    const errorData = JSON.parse(err.message);
    console.log(`✅ Caught expected config error:`);
    console.log(`   Kind: ${errorData.kind}`);
    console.log(`   Message: ${errorData.message}\n`);
  } catch {
    console.log(`✅ Caught expected config error: ${err.message}\n`);
  }
}

// Test 4: Invalid output format
console.log('📦 Test 4: Invalid enum value handling');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    outputDir: join(__dirname, 'dist/error-4'),
    bundle: true,
    format: 'invalid-format'  // Should be one of the OutputFormat enum values
  };

  const bundler = new Fob(config);
  await bundler.bundle();
  console.error('❌ Should have thrown a type error!');
  process.exit(1);
} catch (err) {
  console.log(`✅ Caught expected type error: ${err.message}\n`);
}

// Test 5: Error serialization format
console.log('📦 Test 5: Error JSON serialization');
try {
  const entryPath = join(__dirname, 'fixtures/nonexistent.js');
  const outputDir = join(__dirname, 'dist/error-5');

  await bundleSingle(entryPath, outputDir);
  console.error('❌ Should have thrown an error!');
  process.exit(1);
} catch (err) {
  try {
    const errorData = JSON.parse(err.message);
    console.log(`✅ Error is properly serialized as JSON:`);
    console.log(`   {`);
    console.log(`     kind: "${errorData.kind}",`);
    console.log(`     message: "${errorData.message}",`);
    if (errorData.file) console.log(`     file: "${errorData.file}",`);
    if (errorData.line !== undefined) console.log(`     line: ${errorData.line},`);
    if (errorData.column !== undefined) console.log(`     column: ${errorData.column},`);
    if (errorData.help) console.log(`     help: "${errorData.help}"`);
    console.log(`   }\n`);
  } catch (parseErr) {
    console.error('❌ Error not in expected JSON format:', err.message);
    process.exit(1);
  }
}

console.log('✨ All error handling tests passed!\n');
