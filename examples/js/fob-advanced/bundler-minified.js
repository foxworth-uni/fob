/**
 * Example of building with minification
 *
 * This demonstrates:
 * - Production build with minification
 * - Using the Fob class for reusable bundler instances
 * - Comparing minified vs unminified sizes
 */

import pkg from '@fox-uni/fob';
const { Fob } = pkg;
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

console.log('🔨 Building production bundle with minification...\n');

try {
  // Create a bundler instance with default options
  const bundler = new Fob({
    defaultOptions: {
      entries: [join(__dirname, 'src/index.js')],
      outputDir: join(__dirname, 'dist-prod'),
      platform: 'node',
      format: 'Esm',
    },
  });

  // Build without minification
  console.log('📦 Building without minification...');
  const unminified = await bundler.bundle();

  const unminifiedSize = unminified.stats.totalSize;
  console.log(`Size: ${(unminifiedSize / 1024).toFixed(2)} KB\n`);

  // Build with minification
  console.log('📦 Building with minification...');

  // Create new bundler with minification enabled
  const prodBundler = new Fob({
    defaultOptions: {
      entries: [join(__dirname, 'src/index.js')],
      outputDir: join(__dirname, 'dist-prod'),
      platform: 'node',
      format: 'Esm',
      minify: true,
      sourceMaps: 'external',
    },
  });

  const minified = await prodBundler.bundle();

  const minifiedSize = minified.stats.totalSize;
  const savings = ((1 - minifiedSize / unminifiedSize) * 100).toFixed(1);

  console.log(`   Size: ${(minifiedSize / 1024).toFixed(2)} KB`);
  console.log(`   Savings: ${savings}%\n`);

  console.log('✅ Production build complete!');
  console.log('✨ Output written to dist-prod/');
} catch (error) {
  console.error('❌ Build failed:', error.message);
  process.exit(1);
}
