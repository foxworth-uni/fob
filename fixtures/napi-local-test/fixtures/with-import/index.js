import { add, multiply } from './utils.js';

export function calculate(a, b) {
  return {
    sum: add(a, b),
    product: multiply(a, b)
  };
}
