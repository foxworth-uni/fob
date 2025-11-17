export function normalizeBundleStats(result: unknown): void {
  if (typeof result !== 'object' || result === null) {
    return;
  }

  const statsContainer = result as { stats?: unknown };
  if (typeof statsContainer.stats !== 'object' || statsContainer.stats === null) {
    return;
  }

  const stats = statsContainer.stats as Record<string, unknown>;
  const ensureAlias = (camel: string, snake: string) => {
    if (stats[camel] === undefined && stats[snake] !== undefined) {
      stats[camel] = stats[snake];
    }
  };

  ensureAlias('totalModules', 'total_modules');
  ensureAlias('totalChunks', 'total_chunks');
  ensureAlias('totalSize', 'total_size');
  ensureAlias('durationMs', 'duration_ms');
  ensureAlias('cacheHitRate', 'cache_hit_rate');
}
