import type { EdgeRuntime } from '../runtime/index.js';
import { installFetchRuntime } from '../runtime/index.js';
import type { FetchFileSystemOptions } from '../runtime/fetch-fs.js';

export interface DenoDeployRuntimeOptions {
  baseUrl?: string;
  preload?: FetchFileSystemOptions['preload'];
  fetcher?: typeof fetch;
}

export function createDenoDeployRuntime(options: DenoDeployRuntimeOptions = {}): EdgeRuntime {
  return installFetchRuntime({
    baseUrl: options.baseUrl,
    preload: options.preload,
    fetcher: options.fetcher,
  });
}
