/**
 * Miniflare integration tests for @fob/edge
 * Tests bundler running inside a Worker-like isolate
 *
 * Note: These tests demonstrate the Worker integration pattern.
 * For actual Miniflare testing, you need a proper build setup with
 * bundled worker code since Miniflare doesn't resolve node_modules.
 */

import { test, expect } from 'vitest';

// Skipping Miniflare tests in this setup - they require a bundled worker
// See the unit-node.test.js for functional tests that work without bundling

test.skip('Miniflare integration requires bundled worker', () => {
  expect(true, 'See unit-node.test.js for working tests').toBeTruthy();
});
