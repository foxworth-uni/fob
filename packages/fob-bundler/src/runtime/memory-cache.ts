export class MemoryCache {
  private readonly store = new Map<string, Uint8Array>();

  getSync(key: string): Uint8Array | null {
    const value = this.store.get(key);
    return value ? value.slice() : null;
  }

  setSync(key: string, value: Uint8Array): void {
    this.store.set(key, value.slice());
  }

  deleteSync(key: string): void {
    this.store.delete(key);
  }

  clearSync(): void {
    this.store.clear();
  }
}
