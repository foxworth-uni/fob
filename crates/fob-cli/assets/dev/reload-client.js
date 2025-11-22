/**
 * Fob Dev Server - Auto-reload Client
 *
 * This script connects to the development server via Server-Sent Events (SSE)
 * and automatically reloads the page when a successful build completes.
 *
 * Features:
 * - Automatic reconnection with exponential backoff
 * - Build status notifications in console
 * - Graceful handling of server restarts
 */

(function () {
  'use strict';

  const SSE_ENDPOINT = '/__fob_sse__';
  const MAX_RETRY_DELAY = 30000; // 30 seconds max
  const INITIAL_RETRY_DELAY = 1000; // 1 second initial

  let eventSource = null;
  let retryDelay = INITIAL_RETRY_DELAY;
  let retryTimeout = null;

  /**
   * Connect to SSE endpoint
   */
  function connect() {
    // Clear any pending retry
    if (retryTimeout) {
      clearTimeout(retryTimeout);
      retryTimeout = null;
    }

    console.log('[Fob] Connecting to dev server...');

    try {
      eventSource = new EventSource(SSE_ENDPOINT);

      eventSource.addEventListener('open', () => {
        console.log('[Fob] Connected to dev server');
        // Reset retry delay on successful connection
        retryDelay = INITIAL_RETRY_DELAY;
      });

      eventSource.addEventListener('message', (event) => {
        try {
          const data = JSON.parse(event.data);
          handleEvent(data);
        } catch (e) {
          console.error('[Fob] Failed to parse event:', e);
        }
      });

      eventSource.addEventListener('error', (_event) => {
        console.warn('[Fob] Connection lost, reconnecting...');
        eventSource.close();
        scheduleReconnect();
      });
    } catch (e) {
      console.error('[Fob] Failed to create EventSource:', e);
      scheduleReconnect();
    }
  }

  /**
   * Schedule reconnection with exponential backoff
   */
  function scheduleReconnect() {
    if (retryTimeout) {
      return; // Already scheduled
    }

    console.log(`[Fob] Reconnecting in ${retryDelay}ms...`);

    retryTimeout = setTimeout(() => {
      retryTimeout = null;
      connect();
    }, retryDelay);

    // Exponential backoff with max limit
    retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY);
  }

  /**
   * Handle SSE event from dev server
   */
  function handleEvent(data) {
    switch (data.type) {
      case 'BuildStarted':
        console.log('[Fob] Build started...');
        break;

      case 'BuildCompleted':
        console.log(`[Fob] Build completed in ${data.duration_ms}ms`);
        console.log('[Fob] Reloading page...');

        // Reload the page
        window.location.reload();
        break;

      case 'BuildFailed':
        console.error('[Fob] Build failed:', data.error);
        // Error overlay will be shown by the server
        break;

      case 'ClientConnected':
        // Server acknowledged our connection
        break;

      case 'ClientDisconnected':
        // Another client disconnected
        break;

      default:
        console.log('[Fob] Unknown event:', data);
    }
  }

  /**
   * Cleanup on page unload
   */
  function cleanup() {
    if (retryTimeout) {
      clearTimeout(retryTimeout);
      retryTimeout = null;
    }
    if (eventSource) {
      eventSource.close();
      eventSource = null;
    }
  }

  // Initialize connection
  connect();

  // Cleanup on page unload
  window.addEventListener('beforeunload', cleanup);
  window.addEventListener('unload', cleanup);
})();
