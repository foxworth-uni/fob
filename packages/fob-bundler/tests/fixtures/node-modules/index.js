/**
 * Entry point that imports from node_modules
 */

import { helper } from 'fake-package';

export function useHelper(value) {
  return helper(value);
}
