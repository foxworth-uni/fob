/**
 * Setup script that installs native stub BEFORE any imports
 * Must be imported first in test files that need native stubbing
 */

import { createRequire } from 'node:module';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const fixtureRequire = createRequire(import.meta.url);
const fixturePath = join(__dirname, '../fixtures/index.node.cjs');
const Module = fixtureRequire('module');

// Install the .node extension IMMEDIATELY
Module._extensions['.node'] = (module, _filename) => {
  const fixtureExports = fixtureRequire(fixturePath);
  module.exports = fixtureExports;
};

// Export helpers
export function getNativeMock() {
  return fixtureRequire(fixturePath);
}
