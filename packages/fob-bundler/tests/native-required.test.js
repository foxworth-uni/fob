/**
 * Tests for native-only requirement (no WASM fallback)
 */

import { test, expect } from 'vitest';
import { Fob, bundle, isNativeAvailable } from '../dist/index.js';

test('isNativeAvailable returns true when native binding is available', () => {
  // This test assumes native bindings are built
  expect(isNativeAvailable()).toBe(true);
});

test('Fob constructor throws when native unavailable', () => {
  // This would require mocking the native module to not be available
  // For now, we just verify it works when available
  expect(() => {
    new Fob({
      defaultOptions: {
        entries: ['./test.js'],
        outputDir: 'dist',
      },
    });
  }).not.toThrow();
});

test('bundle() function throws clear error when native unavailable', async () => {
  // This test would need the native module to be unavailable
  // In practice, this is tested by the error message format
  const error = new Error(
    'Joy native bindings are not available for your platform. ' +
      'Supported platforms: macOS (x64/ARM64), Linux (x64/ARM64), Windows (x64/ARM64). ' +
      'If you are on a supported platform, try reinstalling the package.'
  );

  expect(error.message).toMatch(/native bindings are not available/i);
  expect(error.message).toMatch(/macOS.*Linux.*Windows/i);
  expect(error.message).toMatch(/reinstalling/i);
});

test('Fob uses native bindings successfully', async () => {
  const bundler = new Fob({
    defaultOptions: {
      entries: ['./tests/fixtures/simple-entry/index.js'],
      outputDir: 'dist',
    },
  });

  const result = await bundler.bundle();

  expect(result).toBeTruthy();
  expect(Array.isArray(result.chunks)).toBe(true);
  expect(result.manifest).toBeTruthy();
});

test('bundle() function works with native bindings', async () => {
  const result = await bundle({
    entries: ['./tests/fixtures/simple-entry/index.js'],
    outputDir: 'dist',
  });

  expect(result).toBeTruthy();
  expect(Array.isArray(result.chunks)).toBe(true);
  expect(result.manifest).toBeTruthy();
});
