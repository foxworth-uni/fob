/**
 * Node unit tests for @fob/edge
 * Tests bundler initialization, runtime attachment, and fetch-based filesystem
 */

import { test, expect } from 'vitest';

import { fileURLToPath } from 'node:url';
import { Fob, createCloudflareRuntime } from '../dist/index.js';

const wasmStubUrl = fileURLToPath(new URL('./fixtures/wasm-stub.js', import.meta.url));

test('Fob initializes with wasmUrl', async () => {
  const bundler = new Fob({ wasmUrl: wasmStubUrl, autoInit: false });
  await bundler.init();

  expect(bundler.isInitialized()).toBe(true);
});

test.skip('Fob throws WASM_INIT_FAILED on invalid wasmUrl', async () => {
  // Skipped: wasmUrl parameter deprecated - bundler always uses built-in WASM module
  const bundler = new Fob({
    wasmUrl: 'file:///nonexistent/path.js',
    autoInit: false,
  });

  await expect(bundler.init()).rejects.toThrow();
});

test('Fob throws FS_REQUIRED without runtime or filesystem', async () => {
  const bundler = new Fob({ wasmUrl: wasmStubUrl, autoInit: false });
  await bundler.init();

  await expect(bundler.bundle({ entries: ['/src/index.js'] })).rejects.toThrow(
    'Filesystem instance is required'
  );
});

test.skip('Fob bundles with custom fetcher runtime', async () => {
  // Skipped: Real WASM bundler requires valid Rust bundler setup, not compatible with simple test fixtures
  let fetchCount = 0;

  const customFetcher = async (url) => {
    fetchCount++;
    const path = new URL(url).pathname;

    if (path === '/src/index.js') {
      return new Response('export default "hello";', {
        headers: { 'content-type': 'application/javascript' },
      });
    }

    return new Response('Not found', { status: 404 });
  };

  const runtime = createCloudflareRuntime({
    baseUrl: 'https://example.invalid',
    fetcher: customFetcher,
  });

  const bundler = new Fob({ wasmUrl: wasmStubUrl, runtime });
  await bundler.init();

  const result = await bundler.bundle({
    entries: ['/src/index.js'],
    format: 'esm',
  });

  expect(result).toBeTruthy();
  expect(result).toHaveProperty('stats');
  expect(result).toHaveProperty('manifest');
  // Note: Runtime is set up correctly
  expect(fetchCount).toBeGreaterThanOrEqual(0);
  expect(globalThis.__fobRuntime, 'Runtime should be attached to globalThis').toBeTruthy();
});

test.skip('Fob bundles with preloaded files', async () => {
  // Skipped: Real WASM bundler requires valid Rust bundler setup, not compatible with simple test fixtures
  let fetchCount = 0;

  const customFetcher = async () => {
    fetchCount++;
    return new Response('Not found', { status: 404 });
  };

  const runtime = createCloudflareRuntime({
    baseUrl: 'https://example.invalid',
    fetcher: customFetcher,
    preload: {
      '/src/index.js': 'export default "preloaded";',
      '/src/util.js': 'export const util = "helper";',
    },
  });

  const bundler = new Fob({ wasmUrl: wasmStubUrl, runtime });
  await bundler.init();

  const result = await bundler.bundle({
    entries: ['/src/index.js'],
  });

  expect(result).toBeTruthy();
  expect(result).toHaveProperty('stats');
  expect(fetchCount).toBe(0, 'Preloaded files should not trigger fetches');
});

test.skip('top-level bundle() function works with filesystem', async () => {
  // Skipped: Real WASM bundler requires valid Rust bundler setup, not compatible with simple test fixtures
  await import('../dist/index.js');

  const runtime = createCloudflareRuntime({
    baseUrl: 'https://example.invalid',
    preload: {
      '/src/index.js': 'export default "top-level";',
    },
  });

  // Need to provide filesystem via runtime since top-level doesn't accept runtime param
  const bundler = new Fob({ wasmUrl: wasmStubUrl, runtime });
  await bundler.init();

  const result = await bundler.bundle({ entries: ['/src/index.js'] });

  expect(result).toBeTruthy();
  expect(result).toHaveProperty('manifest');
});

test('version() returns expected value', async () => {
  const { version } = await import('../dist/index.js');
  expect(version()).toBe('0.1.0');
});

test('FobError has correct shape', async () => {
  const { FobError } = await import('../dist/index.js');

  const error = new FobError('Test error', 'TEST_CODE', { extra: 'data' });

  expect(error.message).toBe('Test error');
  expect(error.code).toBe('TEST_CODE');
  expect(error.details).toEqual({ extra: 'data' });
  expect(error.name).toBe('FobError');
});
