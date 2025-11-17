/**
 * Service Worker entry point
 * Uses Service Worker APIs, no DOM access
 */

import { cacheAssets, getCachedResponse } from './cache-utils.js';

const CACHE_VERSION = 'v1';

// Install event - cache assets
self.addEventListener('install', (event) => {
  event.waitUntil(cacheAssets(CACHE_VERSION, ['/', '/styles.css', '/app.js']));
});

// Fetch event - serve from cache or network
self.addEventListener('fetch', (event) => {
  event.respondWith(getCachedResponse(event.request).catch(() => fetch(event.request)));
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.filter((name) => name !== CACHE_VERSION).map((name) => caches.delete(name))
      );
    })
  );
});

// Message event - handle messages from clients
self.addEventListener('message', (event) => {
  if (event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});
