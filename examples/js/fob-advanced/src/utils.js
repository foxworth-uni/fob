/**
 * Utility functions
 */

export function greet(name) {
  return `Hello, ${name}! 👋`;
}

export function add(a, b) {
  return a + b;
}

export function multiply(a, b) {
  return a * b;
}

export function divide(a, b) {
  if (b === 0) {
    throw new Error('Cannot divide by zero');
  }
  return a / b;
}
