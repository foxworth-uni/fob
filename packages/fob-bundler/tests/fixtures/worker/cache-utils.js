/**
 * Cache utilities for Service Worker
 */

export async function cacheAssets(cacheName, urls) {
  const cache = await caches.open(cacheName);
  return cache.addAll(urls);
}

export async function getCachedResponse(request) {
  const cache = await caches.open('v1');
  const cached = await cache.match(request);

  if (cached) {
    return cached;
  }

  throw new Error('Not in cache');
}

export async function updateCache(cacheName, request, response) {
  const cache = await caches.open(cacheName);
  return cache.put(request, response);
}

export async function deleteFromCache(cacheName, request) {
  const cache = await caches.open(cacheName);
  return cache.delete(request);
}

export async function clearCache(cacheName) {
  return caches.delete(cacheName);
}
