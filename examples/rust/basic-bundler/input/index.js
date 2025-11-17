// Main entry point
// This file imports other modules and demonstrates basic bundling

import { greet, farewell } from './utils.js';
import { APP_NAME, VERSION } from './constants.js';

console.log(`🚀 ${APP_NAME} v${VERSION}`);
console.log('');

// Use imported functions
console.log(greet('World'));
console.log(greet('Fob Bundler'));
console.log('');

// Show some computation
const numbers = [1, 2, 3, 4, 5];
const sum = numbers.reduce((a, b) => a + b, 0);
console.log(`Sum of ${numbers.join(', ')} = ${sum}`);
console.log('');

console.log(farewell());

// Export for potential use as a module
export { APP_NAME, VERSION };
export default function main() {
  return {
    name: APP_NAME,
    version: VERSION,
    timestamp: new Date().toISOString()
  };
}

