// Simple example entry point
export function greet(name) {
  return `Hello, ${name}!`;
}

export function add(a, b) {
  return a + b;
}

// Run some code when loaded
console.log(greet('Fob'));
console.log('2 + 3 =', add(2, 3));
