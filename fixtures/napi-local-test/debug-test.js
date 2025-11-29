import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { Fob, OutputFormat } from '@fox-uni/fob';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('Testing moduleCount...');
try {
  const config = {
    entries: [join(__dirname, 'fixtures/simple/index.js')],
    output_dir: join(__dirname, 'dist/debug'),
    format: OutputFormat.Esm,
    cwd: __dirname,
  };

  const bundler = new Fob(config);
  const result = await bundler.bundle();

  console.log('Result keys:', Object.keys(result));
  console.log('moduleCount:', result.moduleCount);
  console.log('module_count:', result.module_count);
  console.log('stats:', result.stats);
  console.log('stats.totalModules:', result.stats?.totalModules);
  console.log('Full result:', JSON.stringify(result, null, 2));
} catch (err) {
  console.log('Error message:', err.message);
  console.log('Error message type:', typeof err.message);
  try {
    const parsed = JSON.parse(err.message);
    console.log('Parsed error:', JSON.stringify(parsed, null, 2));
  } catch (e) {
    console.log('Could not parse as JSON');
    console.log('Raw error:', err);
  }
}
