/**
 * Basic example of using @fob/bundler programmatically
 * 
 * This demonstrates:
 * - Simple bundling with the bundle() function
 * - Basic configuration options
 * - Reading bundle results
 */

import { bundle, installNodeRuntime } from '@fob/bundler';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Install the Node.js runtime for filesystem access
installNodeRuntime();

console.log('🔨 Building with @fob/bundler...\n');

try {
  const result = await bundle({
    // Entry points to bundle
    entries: [join(__dirname, 'src/index.js')],
    
    // Output directory
    outputDir: join(__dirname, 'dist'),
    
    // Output format (esm or preserve-modules)
    format: 'esm',
    
    // Target platform
    platform: 'node',
    
    // Generate external source maps
    sourceMaps: 'external',
    
    // Enable code splitting for dynamic imports
    codeSplitting: true,
  });

  console.log('✅ Build complete!\n');
  
  // Display build statistics
  console.log('📊 Build Statistics:');
  console.log(`   Modules: ${result.stats.totalModules}`);
  console.log(`   Chunks: ${result.stats.totalChunks}`);
  console.log(`   Total size: ${(result.stats.totalSize / 1024).toFixed(2)} KB`);
  console.log(`   Duration: ${result.stats.durationMs}ms`);
  console.log(`   Cache hit rate: ${(result.stats.cacheHitRate * 100).toFixed(1)}%\n`);
  
  // Display generated chunks
  console.log('📦 Generated Chunks:');
  for (const chunk of result.chunks) {
    const sizeKB = (chunk.size / 1024).toFixed(2);
    const kindEmoji = chunk.kind === 'entry' ? '🚪' : chunk.kind === 'async' ? '⚡' : '🔗';
    console.log(`   ${kindEmoji} ${chunk.fileName} (${sizeKB} KB) - ${chunk.kind}`);
  }
  
  console.log('\n✨ Output written to dist/');
  
} catch (error) {
  console.error('❌ Build failed:', error.message);
  
  // Handle structured errors from Fob
  if (error.details) {
    console.error('\n📋 Error details:', error.details);
  }
  
  process.exit(1);
}

