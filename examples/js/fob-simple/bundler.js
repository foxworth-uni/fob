import pkg from '@fox-uni/fob';
const { Fob } = pkg;

async function main() {
  console.log('🚀 Building with Fob...\n');

  try {
    const bundler = new Fob({
      entries: ['src/index.js'],
      outputDir: 'dist',
      format: 'Esm',
      sourcemap: 'external',
    });
    const result = await bundler.bundle();

    // Show build results
    console.log('✅ Build complete!\n');
    console.log('📦 Chunks generated:');
    for (const chunk of result.chunks) {
      console.log(`  - ${chunk.fileName} (${chunk.size} bytes)`);
    }

    console.log('\n📊 Build stats:');
    console.log(`  Modules: ${result.stats.totalModules}`);
    console.log(`  Total size: ${result.stats.totalSize} bytes`);
    console.log(`  Duration: ${result.stats.durationMs}ms`);
  } catch (error) {
    console.error('❌ Build failed:', error.message);
    process.exit(1);
  }
}

main();
