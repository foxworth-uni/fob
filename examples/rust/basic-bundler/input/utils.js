// Utility functions
// These will be bundled into the output

export function greet(name) {
  return `👋 Hello, ${name}!`;
}

export function farewell() {
  return '👋 Goodbye! Thanks for trying the basic bundler example.';
}

export function formatDate(date) {
  return new Date(date).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });
}

export function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
