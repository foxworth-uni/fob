// Utility functions
export const VERSION = '1.0.0';

export function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

export function reverse(str) {
  return str.split('').reverse().join('');
}

export function upper(str) {
  return str.toUpperCase();
}
