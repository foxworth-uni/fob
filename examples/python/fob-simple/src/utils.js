// Utility functions module
export function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1).toLowerCase();
}

export function reverse(str) {
  return str.split('').reverse().join('');
}

export const VERSION = '1.0.0';
