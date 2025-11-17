/**
 * Entry point with external dependencies
 */

// These would be marked as external in bundle options
import { readFile } from 'fs/promises';
import { join } from 'path';

export async function loadConfig(filename) {
  const path = join(process.cwd(), filename);
  const content = await readFile(path, 'utf-8');
  return JSON.parse(content);
}

export const localExport = 'bundled-code';
