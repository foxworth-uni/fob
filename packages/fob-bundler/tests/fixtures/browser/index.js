/**
 * Browser entry point - uses DOM and Web APIs
 */

import { updateUI, attachEventListeners } from './dom-utils.js';
import { fetchUser } from './fetch-client.js';

export async function initApp() {
  // Check if we're in a browser environment
  if (typeof window === 'undefined') {
    throw new Error('This module requires a browser environment');
  }

  updateUI('loading', 'Initializing...');

  try {
    const user = await fetchUser();
    updateUI('content', `Welcome, ${user.name}!`);
    attachEventListeners();
  } catch (error) {
    updateUI('error', `Failed to load: ${error.message}`);
  }
}

// Auto-initialize when DOM is ready
if (typeof document !== 'undefined') {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initApp);
  } else {
    initApp();
  }
}
