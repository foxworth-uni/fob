import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { rmSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Clean ALL test output before running
console.log('🧹 Cleaning test outputs...');
rmSync(join(__dirname, 'dist'), { recursive: true, force: true });
console.log('✨ Clean!\n');

const tests = [
  { name: 'Simple Tests', script: 'test-simple.js' },
  { name: 'Advanced Tests', script: 'test-advanced.js' },
  { name: 'Error Handling', script: 'test-errors.js' },
];

console.log('╔═══════════════════════════════════════════╗');
console.log('║   FOB-NATIVE NAPI BINDING TEST SUITE     ║');
console.log('╚═══════════════════════════════════════════╝\n');

async function runTest(test) {
  return new Promise((resolve) => {
    console.log(`\n${'═'.repeat(50)}`);
    console.log(`Running: ${test.name}`);
    console.log('═'.repeat(50));

    const proc = spawn('node', [test.script], {
      cwd: __dirname,
      stdio: 'inherit',
    });

    proc.on('close', (code) => {
      if (code === 0) {
        resolve({ name: test.name, success: true });
      } else {
        resolve({ name: test.name, success: false, code });
      }
    });
  });
}

// Run all tests sequentially
const results = [];
for (const test of tests) {
  const result = await runTest(test);
  results.push(result);
}

// Print summary
console.log('\n\n╔═══════════════════════════════════════════╗');
console.log('║           TEST SUMMARY                    ║');
console.log('╚═══════════════════════════════════════════╝\n');

let passCount = 0;
let failCount = 0;

for (const result of results) {
  if (result.success) {
    console.log(`✅ ${result.name}`);
    passCount++;
  } else {
    console.log(`❌ ${result.name} (exit code: ${result.code})`);
    failCount++;
  }
}

console.log(`\nTotal: ${passCount} passed, ${failCount} failed\n`);

if (failCount > 0) {
  process.exit(1);
}
