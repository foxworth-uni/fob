/**
 * Web Worker entry point
 * Runs background tasks, no DOM access
 */

// Handle messages from main thread
self.addEventListener('message', async (event) => {
  const { type, data } = event.data;

  switch (type) {
    case 'PROCESS_DATA':
      const result = await processData(data);
      self.postMessage({ type: 'RESULT', result });
      break;

    case 'FETCH_URL':
      try {
        const response = await fetch(data.url);
        const json = await response.json();
        self.postMessage({ type: 'FETCH_SUCCESS', data: json });
      } catch (error) {
        self.postMessage({ type: 'FETCH_ERROR', error: error.message });
      }
      break;

    default:
      self.postMessage({ type: 'ERROR', error: `Unknown message type: ${type}` });
  }
});

async function processData(items) {
  // Simulate heavy computation
  return items.map((item) => ({
    ...item,
    processed: true,
    timestamp: Date.now(),
  }));
}

// Signal ready
self.postMessage({ type: 'READY' });
