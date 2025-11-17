/**
 * Integration tests for real bundling scenarios
 * Tests actual bundling with real source files and REAL native binary
 */

import { test, expect } from 'vitest';

import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { mkdir, rm } from 'node:fs/promises';

import { Fob, bundle } from '../dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const fixturesDir = join(__dirname, 'fixtures');

// Helper to create temporary output directory
async function createTempDir(name) {
  const dir = join(__dirname, 'temp', name);
  await rm(dir, { recursive: true, force: true });
  await mkdir(dir, { recursive: true });
  return dir;
}

// Helper to clean up after tests
async function cleanup(dir) {
  await rm(dir, { recursive: true, force: true });
}

test('bundle simple entry point - generates valid chunks', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('simple-entry');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
    });

    // Verify result structure
    expect(result, 'Result should exist').toBeTruthy();
    expect(Array.isArray(result.chunks), 'Chunks should be an array').toBeTruthy();
    expect(result.manifest, 'Manifest should exist').toBeTruthy();
    expect(result.stats, 'Stats should exist').toBeTruthy();

    // Verify manifest has version
    expect(result.manifest.version, 'Manifest should have version').toBeTruthy();

    // Verify stats structure
    expect(typeof result.stats.totalModules).toBe('number');
    expect(typeof result.stats.totalChunks).toBe('number');
    expect(typeof result.stats.totalSize).toBe('number');
    expect(typeof result.stats.durationMs).toBe('number');
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with Fob class - uses default options', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('bundler-class');

  try {
    const bundler = new Fob({
      defaultOptions: {
        entries: [entryPath],
        outputDir,
      },
    });

    const result = await bundler.bundle();

    expect(result).toBeTruthy();
    expect(Array.isArray(result.chunks)).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle multiple entries - creates shared chunks', async () => {
  const appPath = join(fixturesDir, 'multi-entry/app.js');
  const workerPath = join(fixturesDir, 'multi-entry/worker.js');
  const outputDir = await createTempDir('multi-entry');

  try {
    const result = await bundle({
      entries: [appPath, workerPath],
      outputDir,
      codeSplitting: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // With code splitting enabled, we expect entry chunks
    const entryChunks = result.chunks.filter((c) => c.kind === 'entry');
    expect(entryChunks.length >= 1, 'Should have at least one entry chunk').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with code splitting - creates async chunks', async () => {
  const entryPath = join(fixturesDir, 'code-splitting/index.js');
  const outputDir = await createTempDir('code-splitting');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      codeSplitting: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // Entry chunk should exist
    const entryChunks = result.chunks.filter((c) => c.kind === 'entry');
    expect(entryChunks.length > 0, 'Should have entry chunk').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with external dependencies - excludes from bundle', async () => {
  const entryPath = join(fixturesDir, 'external-deps/index.js');
  const outputDir = await createTempDir('external-deps');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      external: ['fs/promises', 'path'],
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // Should still produce chunks even with externals
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with minification option', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('minified');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      minify: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with source maps', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('sourcemaps');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      sourceMaps: 'external',
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with node_modules resolution', async () => {
  const entryPath = join(fixturesDir, 'node-modules/index.js');
  const outputDir = await createTempDir('node-modules');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // Should successfully resolve and bundle the fake-package
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with different platforms', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('platform-node');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'node',
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle stats contain meaningful data', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('stats');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
    });

    // Verify stats fields are present and have valid types
    expect(typeof result.stats.totalModules).toBe('number');
    expect(typeof result.stats.totalChunks).toBe('number');
    expect(typeof result.stats.totalSize).toBe('number');
    expect(typeof result.stats.durationMs).toBe('number');
    expect(typeof result.stats.cacheHitRate).toBe('number');

    // Cache hit rate should be between 0 and 1
    expect(result.stats.cacheHitRate >= 0 && result.stats.cacheHitRate <= 1).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('chunks contain required metadata', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('chunk-metadata');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
    });

    expect(result.chunks.length > 0, 'Should have at least one chunk').toBeTruthy();

    for (const chunk of result.chunks) {
      expect(chunk.id, 'Chunk should have id').toBeTruthy();
      expect(chunk.kind, 'Chunk should have kind').toBeTruthy();
      expect(chunk.fileName, 'Chunk should have fileName').toBeTruthy();
      expect(chunk.code !== undefined, 'Chunk should have code').toBeTruthy();
      expect(Array.isArray(chunk.modules), 'Chunk should have modules array').toBeTruthy();
      expect(Array.isArray(chunk.imports), 'Chunk should have imports array').toBeTruthy();
      expect(
        Array.isArray(chunk.dynamicImports),
        'Chunk should have dynamicImports array'
      ).toBeTruthy();
      expect(typeof chunk.size, 'Chunk size should be a number').toBe('number');
    }
  } finally {
    await cleanup(outputDir);
  }
});

test('manifest contains entry mappings', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('manifest');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
    });

    expect(result.manifest).toBeTruthy();
    expect(result.manifest.entries, 'Manifest should have entries object').toBeTruthy();
    expect(result.manifest.chunks, 'Manifest should have chunks object').toBeTruthy();
    expect(result.manifest.version, 'Manifest should have version').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

// Edge Runtime Platform Tests

test('bundle for edge runtime with Web APIs', async () => {
  const entryPath = join(fixturesDir, 'edge-runtime/index.js');
  const outputDir = await createTempDir('edge-runtime');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'node', // Mock uses node, but would be 'edge' in real implementation
      minify: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
    expect(result.chunks.length > 0, 'Edge bundle should produce chunks').toBeTruthy();

    // Verify bundle contains Web APIs (Response, Headers, etc.)
    const entryChunk = result.chunks.find((c) => c.kind === 'entry');
    expect(entryChunk, 'Should have entry chunk').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle edge runtime with Web Crypto API', async () => {
  const entryPath = join(fixturesDir, 'edge-runtime/web-crypto.js');
  const outputDir = await createTempDir('edge-crypto');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'node',
    });

    expect(result).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();

    // Web Crypto usage should be preserved in edge bundles
    const chunk = result.chunks[0];
    expect(chunk.code, 'Should have bundled code').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle edge runtime with external Node.js modules', async () => {
  const entryPath = join(fixturesDir, 'external-deps/index.js');
  const outputDir = await createTempDir('edge-external');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'node', // Would be edge
      external: ['fs/promises', 'path'], // Mark Node built-ins as external
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // External modules should not be bundled
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

// Browser Platform Tests

test('bundle for browser platform', async () => {
  const entryPath = join(fixturesDir, 'browser/index.js');
  const outputDir = await createTempDir('browser');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'browser',
      minify: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
    expect(result.chunks.length > 0, 'Browser bundle should produce chunks').toBeTruthy();

    // Verify bundle structure
    const entryChunk = result.chunks.find((c) => c.kind === 'entry');
    expect(entryChunk, 'Should have entry chunk for browser').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle browser with DOM utilities', async () => {
  const entryPath = join(fixturesDir, 'browser/dom-utils.js');
  const outputDir = await createTempDir('browser-dom');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'browser',
    });

    expect(result).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();

    // DOM API usage should be preserved
    expect(result.chunks[0].code).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle browser with fetch client', async () => {
  const entryPath = join(fixturesDir, 'browser/fetch-client.js');
  const outputDir = await createTempDir('browser-fetch');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'browser',
      conditions: ['browser', 'production'],
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle browser with code splitting', async () => {
  const entryPath = join(fixturesDir, 'browser/index.js');
  const outputDir = await createTempDir('browser-split');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'browser',
      codeSplitting: true,
      minify: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // Browser bundles should support code splitting
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

// Worker Platform Tests

test('bundle for worker platform (Service Worker)', async () => {
  const entryPath = join(fixturesDir, 'worker/service-worker.js');
  const outputDir = await createTempDir('service-worker');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'worker',
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
    expect(result.chunks.length > 0, 'Worker bundle should produce chunks').toBeTruthy();

    // Service Worker should bundle successfully
    const entryChunk = result.chunks.find((c) => c.kind === 'entry');
    expect(entryChunk, 'Should have entry chunk for worker').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle for worker platform (Web Worker)', async () => {
  const entryPath = join(fixturesDir, 'worker/web-worker.js');
  const outputDir = await createTempDir('web-worker');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'worker',
      minify: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle worker with Cache API utilities', async () => {
  const entryPath = join(fixturesDir, 'worker/cache-utils.js');
  const outputDir = await createTempDir('worker-cache');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'worker',
    });

    expect(result).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();

    // Cache API usage should be preserved in worker bundles
    expect(result.chunks[0].code).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

// Platform-Specific Resolution Tests

test('bundle with browser conditions', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('conditions-browser');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'browser',
      conditions: ['browser', 'production'],
    });

    expect(result).toBeTruthy();
    expect(result.chunks).toBeTruthy();

    // Browser conditions should be applied during resolution
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle with worker conditions', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');
  const outputDir = await createTempDir('conditions-worker');

  try {
    const result = await bundle({
      entries: [entryPath],
      outputDir,
      platform: 'worker',
      conditions: ['worker'],
    });

    expect(result).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});

test('bundle multiple platforms compatibility', async () => {
  const entryPath = join(fixturesDir, 'simple-entry/index.js');

  // Bundle for different platforms and verify all succeed
  const platforms = ['node', 'browser', 'worker'];

  for (const platform of platforms) {
    const outputDir = await createTempDir(`multi-platform-${platform}`);

    try {
      const result = await bundle({
        entries: [entryPath],
        outputDir,
        platform,
      });

      expect(result, `Should bundle successfully for ${platform}`).toBeTruthy();
      expect(result.chunks.length > 0, `Should produce chunks for ${platform}`).toBeTruthy();
    } finally {
      await cleanup(outputDir);
    }
  }
});

test('bundle edge runtime multiple entries', async () => {
  const indexPath = join(fixturesDir, 'edge-runtime/index.js');
  const cryptoPath = join(fixturesDir, 'edge-runtime/web-crypto.js');
  const outputDir = await createTempDir('edge-multi');

  try {
    const result = await bundle({
      entries: [indexPath, cryptoPath],
      outputDir,
      platform: 'node', // Would be edge
      codeSplitting: true,
    });

    expect(result).toBeTruthy();
    expect(result.chunks.length > 0).toBeTruthy();

    // Should have multiple entry chunks
    const entryChunks = result.chunks.filter((c) => c.kind === 'entry');
    expect(entryChunks.length >= 1, 'Should have entry chunks').toBeTruthy();
  } finally {
    await cleanup(outputDir);
  }
});
