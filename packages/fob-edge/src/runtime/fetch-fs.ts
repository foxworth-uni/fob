import type { FileMetadata, FileSystem } from '../types.js';
import { MemoryFileSystem } from './memory-fs.js';

type Fetcher = (url: string) => Promise<Response>;

export interface FetchFileSystemOptions {
  baseUrl?: string;
  fetcher?: Fetcher;
  preload?: Record<string, string | Uint8Array>;
}

export class FetchFileSystem implements FileSystem {
  private readonly memory: MemoryFileSystem;
  private readonly fetcher: Fetcher;
  private readonly baseUrl: string | null;

  constructor(options: FetchFileSystemOptions = {}) {
    this.memory = new MemoryFileSystem(options.preload);
    this.fetcher = options.fetcher ?? fetch;
    this.baseUrl = options.baseUrl ?? null;
  }

  async read(path: string): Promise<Uint8Array> {
    if (this.memory.exists(path)) {
      return this.memory.read(path);
    }

    const url = this.baseUrl ? new URL(path, this.baseUrl).toString() : path;
    const response = await this.fetcher(url);
    if (!response.ok) {
      throw new Error(`Failed to fetch ${url}: ${response.status} ${response.statusText}`);
    }

    const buffer = new Uint8Array(await response.arrayBuffer());
    await this.memory.write(path, buffer);
    return buffer;
  }

  async write(path: string, content: Uint8Array): Promise<void> {
    await this.memory.write(path, content);
  }

  async metadata(path: string): Promise<FileMetadata> {
    return this.memory.metadata(path);
  }

  exists(path: string): boolean {
    return this.memory.exists(path);
  }
}
