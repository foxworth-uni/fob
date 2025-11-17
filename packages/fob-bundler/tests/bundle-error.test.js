/**
 * Tests for bundle error handling with REAL native binary
 */

import { test, expect } from 'vitest';
import { Fob } from '../dist/index.js';

test('Fob.bundle() rejects when no options provided', async () => {
  const bundler = new Fob(); // No defaultOptions

  await expect(async () => {
    await bundler.bundle(); // No options passed
  }).rejects.toThrow('Bundle options are required');
});

test('Fob.bundle() works with defaultOptions', async () => {
  const bundler = new Fob({
    defaultOptions: {
      entries: ['./tests/fixtures/simple-entry/index.js'],
      outputDir: 'dist',
    },
  });

  const result = await bundler.bundle(); // Should use defaultOptions

  expect(result).toBeTruthy();
  expect(Array.isArray(result.chunks)).toBe(true);
});

test('Fob.bundle() accepts override options', async () => {
  const bundler = new Fob({
    defaultOptions: {
      entries: ['./tests/fixtures/simple-entry/index.js'],
      outputDir: 'dist',
    },
  });

  // Override with different entry and output dir
  const result = await bundler.bundle({
    entries: ['./tests/fixtures/code-splitting/index.js'],
    outputDir: 'out',
  });

  expect(result).toBeTruthy();
  expect(Array.isArray(result.chunks)).toBe(true);
});

test.skip('top-level bundle() error handling (TODO: needs mock improvements)', async () => {
  // TODO: This test requires better mocking to simulate native unavailability
  // For now, error handling is tested via integration tests with real errors
});

test('Fob constructor succeeds with valid options', () => {
  // Test that constructor works with valid options
  expect(() => {
    new Fob({
      defaultOptions: {
        entries: ['./tests/fixtures/simple-entry/index.js'],
        outputDir: 'dist',
      },
    });
  }).not.toThrow();
});

test('native errors are properly propagated', async () => {
  // Test with invalid entry to trigger real error
  const bundler = new Fob({
    defaultOptions: {
      entries: ['./nonexistent-file-that-does-not-exist.js'],
      outputDir: 'dist',
    },
  });

  // Native errors should be thrown directly (no fallback)
  await expect(async () => {
    await bundler.bundle();
  }).rejects.toThrow();
});
