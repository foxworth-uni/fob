import { createRequire } from 'node:module';
import type { BundleOptions, BundleResult } from '../types.js';
import { FobError } from '../types.js';
import { normalizeBundleStats } from '../utils/normalize.js';

const require = createRequire(import.meta.url);

/**
 * Rust BundleConfig structure (matches fob-native/src/lib.rs)
 * This is a simplified subset of BundleOptions
 */
interface BundleConfig {
  entries: string[];
  output_dir?: string;
  format?: 'esm' | 'cjs' | 'iife';
  sourcemap?: boolean;
  cwd?: string;
}

/**
 * Native NAPI binding interface (auto-generated from Rust)
 */
interface NativeBinding {
  Fob: new (config: BundleConfig) => NativeFobInstance;
  bundleSingle: (entry: string, outputDir: string, format?: string) => Promise<BundleResult>;
  version: () => string;
}

interface NativeFobInstance {
  bundle(): Promise<BundleResult>;
}

let binding: NativeBinding | null = null;
let loadError: unknown = null;

try {
  // eslint-disable-next-line import/no-dynamic-require
  binding = require('../../index.node') as NativeBinding;
} catch (error) {
  loadError = error;
  binding = null;
}

function ensureBinding(): NativeBinding {
  if (binding) {
    return binding;
  }
  throw loadError ?? new Error('Native binding not available');
}

/**
 * Convert TypeScript BundleOptions to Rust BundleConfig
 * Note: This is a lossy conversion - many advanced features are not yet supported in Rust
 */
function convertToBundleConfig(options: BundleOptions): BundleConfig {
  // Map format (TypeScript uses different names)
  let format: 'esm' | 'cjs' | 'iife' | undefined;
  if (options.format === 'esm' || options.format === 'preserve-modules') {
    format = 'esm';
  }

  // Map sourceMaps boolean
  const sourcemap = options.sourceMaps ? options.sourceMaps !== 'none' : undefined;

  return {
    entries: options.entries,
    output_dir: options.outputDir, // Rust will default to "dist" if undefined
    format,
    sourcemap,
    // cwd is not in BundleOptions, will use process.cwd()
  };
}

export function isNativeAvailable(): boolean {
  return binding !== null;
}

export class NativeFob {
  private instance: NativeFobInstance | null = null;
  private config: BundleConfig | null = null;

  constructor(private readonly defaultOptions?: BundleOptions) {
    if (defaultOptions) {
      this.config = convertToBundleConfig(defaultOptions);
    }
  }

  async bundle(options?: BundleOptions): Promise<BundleResult> {
    const native = ensureBinding();

    // Use provided options or fall back to constructor options
    const config = options ? convertToBundleConfig(options) : this.config;
    if (!config) {
      throw new Error('Bundle options are required');
    }

    // Create new instance or reuse existing
    if (!this.instance || options) {
      this.instance = new native.Fob(config);
    }

    try {
      // NAPI returns BundleResult directly - no JSON parsing needed
      const result = await this.instance.bundle();
      normalizeBundleStats(result);
      return result;
    } catch (error) {
      // Rust errors are thrown as exceptions
      if (error instanceof Error) {
        throw new FobError(error.message);
      }
      throw new FobError(String(error));
    }
  }
}

export async function bundle(options: BundleOptions): Promise<BundleResult> {
  const native = ensureBinding();
  const config = convertToBundleConfig(options);
  const fob = new native.Fob(config);

  try {
    // NAPI returns BundleResult directly - no JSON parsing needed
    const result = await fob.bundle();
    normalizeBundleStats(result);
    return result;
  } catch (error) {
    // Rust errors are thrown as exceptions
    if (error instanceof Error) {
      throw new FobError(error.message);
    }
    throw new FobError(String(error));
  }
}

/**
 * Quick helper to bundle a single entry point
 */
export async function bundleSingle(
  entry: string,
  outputDir: string = 'dist',
  format?: 'esm' | 'cjs' | 'iife'
): Promise<BundleResult> {
  const native = ensureBinding();

  try {
    const result = await native.bundleSingle(entry, outputDir, format);
    normalizeBundleStats(result);
    return result;
  } catch (error) {
    if (error instanceof Error) {
      throw new FobError(error.message);
    }
    throw new FobError(String(error));
  }
}

/**
 * Get the bundler version
 */
export function version(): string {
  const native = ensureBinding();
  return native.version();
}
