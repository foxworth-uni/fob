/**
 * Example of a simple file watcher with rebuild
 * 
 * This demonstrates:
 * - Watching for file changes
 * - Rebuilding on changes
 * - Error handling during development
 */

import { bundle } from '@fob/bundler';
import { watch } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const srcDir = join(__dirname, 'src');
const outputDir = join(__dirname, 'dist-watch');

let isBuilding = false;
let needsRebuild = false;

async function build() {
  if (isBuilding) {
    needsRebuild = true;
    return;
  }
  
  isBuilding = true;
  needsRebuild = false;
  
  const startTime = Date.now();
  console.log(`\n🔨 Building... [${new Date().toLocaleTimeString()}]`);
  
  try {
    const result = await bundle({
      entries: [join(srcDir, 'index.js')],
      outputDir,
      format: 'esm',
      platform: 'node',
      sourceMaps: 'inline',
      codeSplitting: true,
    });
    
    const duration = Date.now() - startTime;
    console.log(`✅ Built in ${duration}ms (${result.stats.totalChunks} chunks, ${(result.stats.totalSize / 1024).toFixed(2)} KB)`);
    
  } catch (error) {
    console.error('❌ Build failed:', error.message);
    
    if (error.details) {
      console.error('   Type:', error.details.type);
    }
  } finally {
    isBuilding = false;
    
    // If a change occurred during build, rebuild
    if (needsRebuild) {
      setTimeout(build, 100);
    }
  }
}

console.log('👀 Watching src/ for changes...');
console.log('   Press Ctrl+C to stop\n');

// Initial build
await build();

// Watch for changes
const watcher = watch(srcDir, { recursive: true }, (eventType, filename) => {
  if (filename && filename.endsWith('.js')) {
    console.log(`📝 Changed: ${filename}`);
    build();
  }
});

// Cleanup on exit
process.on('SIGINT', () => {
  console.log('\n\n👋 Stopping watcher...');
  watcher.close();
  process.exit(0);
});

