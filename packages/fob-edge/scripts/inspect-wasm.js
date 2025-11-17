#!/usr/bin/env node
/**
 * Inspect WASM module exports
 * 
 * This script loads the WASM module and lists all exports to help
 * understand the wit-bindgen function name mangling.
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const wasmPath = join(__dirname, '../wasm/bundler/fob_bundler_wasm.wasm');

console.log('Loading WASM module:', wasmPath);
console.log('');

try {
  const wasmBytes = readFileSync(wasmPath);
  const wasmModule = new WebAssembly.Module(wasmBytes);
  
  // Get all exports
  const exports = WebAssembly.Module.exports(wasmModule);
  const imports = WebAssembly.Module.imports(wasmModule);
  
  console.log('=== EXPORTS ===');
  console.log(`Total exports: ${exports.length}`);
  console.log('');
  
  // Categorize exports
  const functions = exports.filter(e => e.kind === 'function');
  const memory = exports.filter(e => e.kind === 'memory');
  const tables = exports.filter(e => e.kind === 'table');
  const globals = exports.filter(e => e.kind === 'global');
  
  console.log('Functions:');
  functions.forEach(exp => {
    console.log(`  - ${exp.name}`);
  });
  
  console.log('');
  console.log('Memory exports:', memory.map(e => e.name).join(', '));
  console.log('Table exports:', tables.map(e => e.name).join(', '));
  console.log('Global exports:', globals.map(e => e.name).join(', '));
  
  console.log('');
  console.log('=== IMPORTS ===');
  console.log(`Total imports: ${imports.length}`);
  console.log('');
  
  // Group imports by module
  const importsByModule = {};
  imports.forEach(imp => {
    if (!importsByModule[imp.module]) {
      importsByModule[imp.module] = [];
    }
    importsByModule[imp.module].push(`${imp.name} (${imp.kind})`);
  });
  
  Object.entries(importsByModule).forEach(([module, items]) => {
    console.log(`Module: "${module}"`);
    items.forEach(item => {
      console.log(`  - ${item}`);
    });
    console.log('');
  });
  
  console.log('');
  console.log('=== WIT-RELATED EXPORTS ===');
  const witRelated = functions.filter(f => 
    f.name.includes('fob') ||
    f.name.includes('bundle') ||
    f.name.includes('cabi') ||
    f.name.includes('runtime') ||
    f.name.includes('version')
  );
  
  if (witRelated.length > 0) {
    console.log('Potential WIT-generated functions:');
    witRelated.forEach(exp => {
      console.log(`  ✓ ${exp.name}`);
    });
  } else {
    console.log('No obvious WIT-related exports found.');
    console.log('');
    console.log('All function exports:');
    functions.slice(0, 20).forEach(exp => {
      console.log(`  - ${exp.name}`);
    });
    if (functions.length > 20) {
      console.log(`  ... and ${functions.length - 20} more`);
    }
  }
  
} catch (error) {
  console.error('Error inspecting WASM module:', error.message);
  process.exit(1);
}

