import type { BundleOptions, BundleResult } from './types.js';
import * as native from './native/index.js';
export type { BundleOptions, BundleResult } from './types.js';
export { isNativeAvailable } from './native/index.js';
export {
  DiskCache,
  installMemoryRuntime,
  installNodeRuntime,
  MemoryCache,
  MemoryFileSystem,
  NativeFileSystem,
} from './runtime/index.js';
export type { InstallNodeRuntimeOptions } from './runtime/index.js';
export { FobError } from './types.js';

export interface NodeFobOptions {
  defaultOptions?: BundleOptions;
}

function throwNativeUnavailable(): never {
  throw new Error(
    'Fob native bindings are not available for your platform. ' +
      'Supported platforms: macOS (x64/ARM64), Linux (x64/ARM64), Windows (x64/ARM64). ' +
      'If you are on a supported platform, try reinstalling the package.'
  );
}

export class Fob {
  private readonly defaultOptions?: BundleOptions;
  private readonly nativeInstance?: native.NativeFob;

  constructor(options: NodeFobOptions = {}) {
    this.defaultOptions = options.defaultOptions;

    if (!native.isNativeAvailable()) {
      throwNativeUnavailable();
    }

    this.nativeInstance = new native.NativeFob(this.defaultOptions);
  }

  async bundle(options?: BundleOptions): Promise<BundleResult> {
    const config = options ?? this.defaultOptions;
    if (!this.nativeInstance) {
      throwNativeUnavailable();
    }

    return this.nativeInstance.bundle(config);
  }
}

export async function bundle(options: BundleOptions): Promise<BundleResult> {
  if (!native.isNativeAvailable()) {
    throwNativeUnavailable();
  }
  return await native.bundle(options);
}

export async function bundleInMemory(
  files: Record<string, string | Uint8Array>,
  options: BundleOptions
): Promise<BundleResult> {
  if (!native.isNativeAvailable()) {
    throwNativeUnavailable();
  }

  const { installMemoryRuntime } = await import('./runtime/index.js');
  installMemoryRuntime(files);
  return await native.bundle(options);
}

export function version(): string {
  if (!native.isNativeAvailable()) {
    throwNativeUnavailable();
  }
  return native.version();
}

// Config helpers for programmatic use
export { createConfig, validateConfig } from './config.js';
export type { PartialBundleOptions } from './config.js';
