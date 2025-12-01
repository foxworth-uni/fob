import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // Use node environment for WASM file loading
    environment: 'node',

    // Test file patterns
    include: ['src/**/*.test.ts', 'src/**/*.spec.ts'],

    // Coverage configuration
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['src/**/*.ts'],
      exclude: ['src/**/*.test.ts', 'src/**/*.spec.ts'],
    },

    // Globals - make test functions available without imports
    globals: true,

    // Test timeout (WASM initialization can take a moment)
    testTimeout: 10000,
  },
});
