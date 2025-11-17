/**
 * Tests for version() function with REAL native binary
 */

import { test, expect } from 'vitest';
import { version, isNativeAvailable } from '../dist/index.js';

test('version() returns native version when available', () => {
  expect(isNativeAvailable()).toBe(true);

  const ver = version();
  // Real native binary returns actual cargo version
  expect(typeof ver).toBe('string');
  expect(ver).toMatch(/^\d+\.\d+\.\d+/);
});

test('version() returns string format', () => {
  const ver = version();

  // Version should be a string in semver format
  expect(typeof ver).toBe('string');
  expect(ver).toMatch(/^\d+\.\d+\.\d+/);
});

test('version() can be called multiple times', () => {
  const ver1 = version();
  const ver2 = version();

  // Should return same version consistently
  expect(typeof ver1).toBe('string');
  expect(typeof ver2).toBe('string');
  expect(ver1).toBe(ver2);
});
