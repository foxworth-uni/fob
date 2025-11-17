/**
 * First entry point - app
 */

import { shared } from './shared.js';

export function startApp() {
  return `App started: ${shared()}`;
}
