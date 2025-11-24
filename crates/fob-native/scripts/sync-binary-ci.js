#!/usr/bin/env node
/**
 * CI-specific binary sync script.
 * More lenient than the regular sync script - provides diagnostics and
 * handles cases where node_modules may not exist yet.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

// Get target from environment variable (for CI) or detect from host
const targetFromEnv = process.env.NAPI_TARGET;
let binaryName;

if (targetFromEnv) {
  // Parse Rust target triple (e.g., "x86_64-apple-darwin" -> "darwin-x64")
  if (targetFromEnv.includes('x86_64-apple-darwin')) {
    binaryName = 'fob-native.darwin-x64.node';
  } else if (targetFromEnv.includes('aarch64-apple-darwin')) {
    binaryName = 'fob-native.darwin-arm64.node';
  } else if (targetFromEnv.includes('x86_64-pc-windows-msvc')) {
    binaryName = 'fob-native.win32-x64-msvc.node';
  } else if (targetFromEnv.includes('x86_64-unknown-linux-gnu')) {
    binaryName = 'fob-native.linux-x64-gnu.node';
  } else if (targetFromEnv.includes('x86_64-unknown-linux-musl')) {
    binaryName = 'fob-native.linux-x64-musl.node';
  } else if (targetFromEnv.includes('aarch64-unknown-linux-gnu')) {
    binaryName = 'fob-native.linux-arm64-gnu.node';
  } else if (targetFromEnv.includes('aarch64-unknown-linux-musl')) {
    binaryName = 'fob-native.linux-arm64-musl.node';
  } else {
    console.error(`Unsupported target: ${targetFromEnv}`);
    process.exit(1);
  }
} else {
  // Fallback to host detection (for local development)
  const platform = os.platform();
  const arch = os.arch();
  
  if (platform === 'darwin') {
    const darwinArch = arch === 'arm64' ? 'arm64' : 'x64';
    binaryName = `fob-native.darwin-${darwinArch}.node`;
  } else if (platform === 'linux') {
    const linuxArch = arch === 'arm64' ? 'arm64' : 'x64';
    binaryName = `fob-native.linux-${linuxArch}-gnu.node`;
  } else if (platform === 'win32') {
    binaryName = `fob-native.win32-x64-msvc.node`;
  } else {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
  }
}

const sourcePath = path.join(__dirname, '..', binaryName);

console.log('=== CI Binary Sync Diagnostics ===');
if (targetFromEnv) {
  console.log(`Target (from env): ${targetFromEnv}`);
} else {
  console.log(`Platform: ${os.platform()}, Arch: ${os.arch()}`);
}
console.log(`Expected binary: ${binaryName}`);
console.log(`Looking for: ${sourcePath}`);
console.log('');

// List all .node files in the directory
console.log('Available .node files in crates/fob-native/:');
const nativeDir = path.join(__dirname, '..');
try {
  const files = fs.readdirSync(nativeDir);
  const nodeFiles = files.filter(f => f.endsWith('.node'));
  if (nodeFiles.length === 0) {
    console.log('  ⚠️  No .node files found!');
    console.log('');
    console.log('This likely means:');
    console.log('  1. The build step failed');
    console.log('  2. Artifacts were not downloaded correctly');
    console.log('  3. The artifact path is incorrect');
    process.exit(1);
  } else {
    nodeFiles.forEach(f => {
      const stat = fs.statSync(path.join(nativeDir, f));
      console.log(`  ✓ ${f} (${(stat.size / 1024).toFixed(2)} KB)`);
    });
  }
} catch (err) {
  console.error(`Error reading directory: ${err.message}`);
  process.exit(1);
}
console.log('');

// Check if expected binary exists
if (!fs.existsSync(sourcePath)) {
  console.error(`❌ Error: Expected binary not found!`);
  console.error(`   Path: ${sourcePath}`);
  console.error('');
  console.error('Available binaries do not match the current platform/arch.');
  console.error('This is expected if artifacts from a different platform were downloaded.');
  process.exit(1);
}

console.log(`✓ Found expected binary: ${binaryName}`);
console.log('');

// Find all potential node_modules locations
const rootDir = path.resolve(__dirname, '../../..');
const targets = [];

const findPnpmLocations = (dir, depth = 0) => {
  if (depth > 5) return;
  
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      
      if (entry.isDirectory()) {
        if (entry.name.startsWith('fob-native-build@')) {
          const packageDir = path.join(fullPath, 'node_modules', 'fob-native-build');
          if (fs.existsSync(packageDir)) {
            targets.push(packageDir);
          }
        } else if (entry.name === 'node_modules' && depth === 0) {
          const pnpmDir = path.join(fullPath, '.pnpm');
          if (fs.existsSync(pnpmDir)) {
            findPnpmLocations(pnpmDir, depth + 1);
          }
        } else if (entry.name === '.pnpm') {
          findPnpmLocations(fullPath, depth + 1);
        }
      }
    }
  } catch (err) {
    // Ignore permission errors
  }
};

// Search from root
console.log('Searching for node_modules locations...');
findPnpmLocations(rootDir);

// Also check fixtures directory
const fixturesDir = path.join(rootDir, 'fixtures');
if (fs.existsSync(fixturesDir)) {
  findPnpmLocations(fixturesDir);
}

const uniqueTargets = [...new Set(targets.map(t => path.resolve(t)))];

if (uniqueTargets.length === 0) {
  console.warn('⚠️  No node_modules locations found.');
  console.warn('This is normal if:');
  console.warn('  - Dependencies have not been installed yet');
  console.warn('  - Running in a fresh CI environment');
  console.warn('');
  console.warn('Binary is available but not synced to node_modules.');
  console.warn('Tests may fail if they depend on the synced binary.');
  process.exit(0);
}

console.log(`Found ${uniqueTargets.length} target location(s):`);
uniqueTargets.forEach(t => console.log(`  - ${path.relative(rootDir, t)}`));
console.log('');

// Copy binary to all targets
let successCount = 0;
let failCount = 0;

console.log('Syncing binary...');
for (const targetDir of uniqueTargets) {
  const targetPath = path.join(targetDir, binaryName);
  
  try {
    fs.mkdirSync(targetDir, { recursive: true });
    fs.copyFileSync(sourcePath, targetPath);
    console.log(`  ✓ ${path.relative(rootDir, targetPath)}`);
    successCount++;
  } catch (err) {
    console.error(`  ✗ ${path.relative(rootDir, targetPath)}: ${err.message}`);
    failCount++;
  }
}

console.log('');
console.log(`=== Sync Results ===`);
console.log(`Success: ${successCount}`);
console.log(`Failed: ${failCount}`);

if (failCount > 0) {
  process.exit(1);
}

