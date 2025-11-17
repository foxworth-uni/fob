import type { EdgeRuntime } from '../runtime/index.js';
import { installFetchRuntime } from '../runtime/index.js';
import type { FetchFileSystemOptions } from '../runtime/fetch-fs.js';

export interface CloudflareRuntimeOptions {
  baseUrl?: string;
  preload?: FetchFileSystemOptions['preload'];
  fetcher?: (input: RequestInfo, init?: RequestInit) => Promise<Response>;
}

export function createCloudflareRuntime(options: CloudflareRuntimeOptions = {}): EdgeRuntime {
  return installFetchRuntime({
    baseUrl: options.baseUrl,
    preload: options.preload,
    fetcher: options.fetcher,
  });
}
