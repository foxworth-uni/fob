// Main application entry point
// Demonstrates path aliases and shared code

import { add, multiply, divide } from '@lib/math';
import { formatNumber, formatCurrency } from '@lib/format';
import { APP_CONFIG } from './config.js';

console.log(`🚀 ${APP_CONFIG.name} v${APP_CONFIG.version}`);
console.log('');

// Math operations
const a = 10;
const b = 5;

console.log('📊 Math Operations:');
console.log(`   ${a} + ${b} = ${add(a, b)}`);
console.log(`   ${a} × ${b} = ${multiply(a, b)}`);
console.log(`   ${a} ÷ ${b} = ${divide(a, b)}`);
console.log('');

// Formatting
const price = 1234.56;
console.log('💰 Formatting:');
console.log(`   Number: ${formatNumber(price)}`);
console.log(`   Currency: ${formatCurrency(price)}`);
console.log('');

// Export for use as a module
export function calculate(x, y, operation) {
  switch (operation) {
    case 'add':
      return add(x, y);
    case 'multiply':
      return multiply(x, y);
    case 'divide':
      return divide(x, y);
    default:
      throw new Error(`Unknown operation: ${operation}`);
  }
}

export { APP_CONFIG };
