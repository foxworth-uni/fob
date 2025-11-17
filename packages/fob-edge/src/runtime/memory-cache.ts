export class MemoryCache {
  private readonly store = new Map<string, Uint8Array>();
  private hits = 0;
  private misses = 0;

  get(key: string): Uint8Array | null {
    const value = this.store.get(key);
    if (value) {
      this.hits += 1;
      return value.slice();
    }
    this.misses += 1;
    return null;
  }

  set(key: string, value: Uint8Array): void {
    this.store.set(key, value.slice());
  }

  delete(key: string): void {
    this.store.delete(key);
  }

  clear(): void {
    this.store.clear();
    this.hits = 0;
    this.misses = 0;
  }

  hitRate(): number {
    const total = this.hits + this.misses;
    return total === 0 ? 0 : this.hits / total;
  }
}
