import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['tests/**/*.test.{js,ts}'],
    environment: 'node', // Use node for edge runtime (or custom environment if available)
    globals: false,
    testTimeout: 60_000,
  },
});
