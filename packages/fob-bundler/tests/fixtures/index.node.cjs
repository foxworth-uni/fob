/**
 * Mock native binding fixture for testing
 * Simulates the .node native module behavior
 */

let mockState = {
  // Wrapped in BundleTaskResult format with discriminated union
  response: {
    status: 'success',
    result: {
      chunks: [
        {
          id: 'main-abc123',
          kind: 'entry',
          fileName: 'main-abc123.js',
          code: 'export default function main() { return "bundled"; }',
          modules: [
            {
              path: './index.js',
              size: 150,
              hasSideEffects: false,
            },
          ],
          imports: [],
          dynamicImports: [],
          size: 150,
        },
      ],
      manifest: {
        entries: {
          './index.js': 'main-abc123.js',
        },
        chunks: {
          'main-abc123': {
            file: 'main-abc123.js',
            imports: [],
            dynamicImports: [],
          },
        },
        version: 'native-test',
      },
      stats: {
        totalModules: 1,
        totalChunks: 1,
        totalSize: 150,
        durationMs: 42,
        cacheHitRate: 0.0,
      },
      assets: [],
    },
  },
  version: '0.1.0-native',
  shouldThrow: false,
  throwOnVersion: false,
};

class Fob {
  constructor(optionsJSON) {
    this.options = optionsJSON ? JSON.parse(optionsJSON) : null;
  }

  async bundle() {
    if (mockState.shouldThrow) {
      throw new Error('Native bundle error');
    }
    return JSON.stringify(mockState.response);
  }
}

function bundle(_optionsJSON) {
  if (mockState.shouldThrow) {
    throw new Error('Native bundle error');
  }
  return Promise.resolve(JSON.stringify(mockState.response));
}

function version() {
  if (mockState.throwOnVersion) {
    throw new Error('Native version error');
  }
  return mockState.version;
}

function __setNativeMockState(state) {
  Object.assign(mockState, state);
}

function __resetNativeMockState() {
  mockState = {
    response: {
      status: 'success',
      result: {
        chunks: [
          {
            id: 'main-abc123',
            kind: 'entry',
            fileName: 'main-abc123.js',
            code: 'export default function main() { return "bundled"; }',
            modules: [
              {
                path: './index.js',
                size: 150,
                hasSideEffects: false,
              },
            ],
            imports: [],
            dynamicImports: [],
            size: 150,
          },
        ],
        manifest: {
          entries: {
            './index.js': 'main-abc123.js',
          },
          chunks: {
            'main-abc123': {
              file: 'main-abc123.js',
              imports: [],
              dynamicImports: [],
            },
          },
          version: 'native-test',
        },
        stats: {
          totalModules: 1,
          totalChunks: 1,
          totalSize: 150,
          durationMs: 42,
          cacheHitRate: 0.0,
        },
        assets: [],
      },
    },
    version: '0.1.0-native',
    shouldThrow: false,
    throwOnVersion: false,
  };
}

/**
 * Set a custom error response for testing error handling
 * @param {Object} error - The error object matching FobErrorDetails type
 */
function __setErrorResponse(error) {
  mockState.response = {
    status: 'error',
    error,
  };
}

module.exports = {
  Fob,
  bundle,
  version,
  __setNativeMockState,
  __resetNativeMockState,
  __setErrorResponse,
};
