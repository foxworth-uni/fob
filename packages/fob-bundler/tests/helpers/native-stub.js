/**
 * Helper to install/restore .node extension handling for testing
 */

import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const fixtureRequire = createRequire(import.meta.url);
const fixturePath = join(__dirname, '../fixtures/index.node.cjs');

let originalNodeExtension = null;

/**
 * Install the .node extension to load our JS fixture
 */
export function installNativeStub() {
  const Module = fixtureRequire('module');

  // Save original if exists
  if (Module._extensions['.node']) {
    originalNodeExtension = Module._extensions['.node'];
  }

  // Install our stub
  Module._extensions['.node'] = (module, _filename) => {
    const fixtureExports = fixtureRequire(fixturePath);
    module.exports = fixtureExports;
  };
}

/**
 * Restore the original .node extension handler
 */
export function restoreNativeStub() {
  const Module = fixtureRequire('module');

  if (originalNodeExtension) {
    Module._extensions['.node'] = originalNodeExtension;
  } else {
    delete Module._extensions['.node'];
  }

  originalNodeExtension = null;
}

/**
 * Get the mock fixture to control test behavior
 */
export function getNativeMock() {
  return fixtureRequire(fixturePath);
}
