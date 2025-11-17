/**
 * Example application entry point
 */

import { greet, add, multiply } from './utils.js';
import { formatDate } from './date-utils.js';

console.log('🚀 Fob Bundler Example\n');

// Basic functionality
console.log(greet('Developer'));
console.log(`2 + 3 = ${add(2, 3)}`);
console.log(`4 × 5 = ${multiply(4, 5)}`);

// Date formatting
const now = new Date();
console.log(`Current time: ${formatDate(now)}`);

// Dynamic import (will be code-split)
console.log('\n📦 Loading heavy module...');
const heavy = await import('./heavy-module.js');
console.log(heavy.processData([1, 2, 3, 4, 5]));

console.log('\n✅ Example complete!');

