import type { FileMetadata, FileSystem } from '../types.js';

export class MemoryFileSystem implements FileSystem {
  private readonly files = new Map<string, Uint8Array>();
  private readonly metadataMap = new Map<string, FileMetadata>();

  constructor(initialFiles: Record<string, string | Uint8Array> = {}) {
    for (const [path, content] of Object.entries(initialFiles)) {
      this.writeSync(path, content);
    }
  }

  async read(path: string): Promise<Uint8Array> {
    const content = this.files.get(path);
    if (!content) {
      throw new Error(`File not found: ${path}`);
    }
    return content.slice();
  }

  async write(path: string, content: Uint8Array): Promise<void> {
    this.writeSync(path, content);
  }

  private writeSync(path: string, content: string | Uint8Array): void {
    const bytes = typeof content === 'string' ? new TextEncoder().encode(content) : content.slice();
    this.files.set(path, bytes);
    this.metadataMap.set(path, {
      size: bytes.byteLength,
      modifiedMs: Date.now(),
    });
  }

  async metadata(path: string): Promise<FileMetadata> {
    const meta = this.metadataMap.get(path);
    if (!meta) {
      throw new Error(`File not found: ${path}`);
    }
    return { ...meta };
  }

  exists(path: string): boolean {
    return this.files.has(path);
  }

  list(): string[] {
    return [...this.files.keys()];
  }
}
