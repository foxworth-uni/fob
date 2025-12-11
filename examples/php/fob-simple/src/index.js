// Simple example entry point
import { capitalize, reverse, VERSION } from './utils.js';

export function greet(name) {
  return `Hello, ${name}!`;
}

export function add(a, b) {
  return a + b;
}

export function multiply(a, b) {
  return a * b;
}

// Run some code when loaded
console.log(greet('Fob'));
console.log('2 + 3 =', add(2, 3));
console.log('2 * 3 =', multiply(2, 3));
console.log('Capitalized:', capitalize('hello world'));
console.log('Reversed:', reverse('fob'));
console.log('Version:', VERSION);
