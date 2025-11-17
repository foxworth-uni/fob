/**
 * Second entry point - worker
 */

import { shared } from './shared.js';

export function startWorker() {
  return `Worker started: ${shared()}`;
}
