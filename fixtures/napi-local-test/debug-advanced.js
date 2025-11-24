import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { mkdirSync, rmSync, writeFileSync } from 'fs';
import { Fob, OutputFormat, SourceMapMode } from 'fob-native-build';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('📦 Test 3: Inline sourcemaps');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    output_dir: join(__dirname, 'dist/advanced-3'),
    bundle: true,
    format: OutputFormat.Esm,
    cwd: __dirname,
    sourcemap: SourceMapMode.Inline
  };

  rmSync(config.output_dir, { recursive: true, force: true });
  mkdirSync(config.output_dir, { recursive: true });

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log('Result chunks:', result.chunks.length);
  console.log('First chunk fileName:', result.chunks[0]?.fileName);
  console.log('First chunk code length:', result.chunks[0]?.code?.length);
  
  // Manually write the file to test
  if (result.chunks[0]) {
    const outputFile = join(config.output_dir, result.chunks[0].fileName);
    console.log('Writing to:', outputFile);
    writeFileSync(outputFile, result.chunks[0].code);
    console.log('✅ File written successfully');
  }
} catch (err) {
  console.error('❌ Failed:', err.message);
  console.error('Stack:', err.stack);
  process.exit(1);
}

