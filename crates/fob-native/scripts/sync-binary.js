#!/usr/bin/env node
/**
 * Syncs the built NAPI binary to all node_modules locations where it might be loaded.
 *
 * This script handles the case where pnpm's file: protocol copies packages at install time,
 * so rebuilding the binary in the source location doesn't automatically update the copy
 * in node_modules.
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

// Detect platform and architecture
const platform = os.platform();
const arch = os.arch();

// Map to napi binary naming convention: fob-native.{platform}-{arch}.node
// For darwin: darwin-arm64 or darwin-x64
// For linux: linux-x64-gnu, linux-x64-musl, linux-arm64-gnu, linux-arm64-musl
// For windows: win32-x64-msvc
let binaryName;
if (platform === 'darwin') {
  const darwinArch = arch === 'arm64' ? 'arm64' : 'x64';
  binaryName = `fob-native.darwin-${darwinArch}.node`;
} else if (platform === 'linux') {
  // Default to gnu, but could be musl
  const linuxArch = arch === 'arm64' ? 'arm64' : 'x64';
  binaryName = `fob-native.linux-${linuxArch}-gnu.node`;
} else if (platform === 'win32') {
  binaryName = `fob-native.win32-x64-msvc.node`;
} else {
  console.error(`Unsupported platform: ${platform}`);
  process.exit(1);
}
const sourcePath = path.join(__dirname, '..', binaryName);

if (!fs.existsSync(sourcePath)) {
  console.error(`Error: Binary not found at ${sourcePath}`);
  console.error(`Expected binary name: ${binaryName}`);
  console.error(`Platform: ${platform}, Arch: ${arch}`);
  process.exit(1);
}

// Find all potential node_modules locations
const rootDir = path.resolve(__dirname, '../..');
const targets = [];

// 1. Direct node_modules in root
const rootNodeModules = path.join(rootDir, 'node_modules', 'fob-native-build');
if (fs.existsSync(rootNodeModules)) {
  targets.push(rootNodeModules);
}

// 2. pnpm store locations (.pnpm directory)
const findPnpmLocations = (dir, depth = 0) => {
  if (depth > 5) return; // Limit recursion depth

  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        // Check if this is a pnpm package directory (format: package@version)
        if (entry.name.startsWith('fob-native-build@')) {
          // Inside this directory, there should be node_modules/fob-native-build
          const packageDir = path.join(fullPath, 'node_modules', 'fob-native-build');
          if (fs.existsSync(packageDir)) {
            targets.push(packageDir);
          }
        } else if (entry.name === 'node_modules' && depth === 0) {
          // Look in node_modules/.pnpm
          const pnpmDir = path.join(fullPath, '.pnpm');
          if (fs.existsSync(pnpmDir)) {
            findPnpmLocations(pnpmDir, depth + 1);
          }
        } else if (entry.name === '.pnpm') {
          findPnpmLocations(fullPath, depth + 1);
        } else if (depth < 2) {
          // Recursively search subdirectories (but limit depth)
          findPnpmLocations(fullPath, depth + 1);
        }
      }
    }
  } catch (err) {
    // Ignore permission errors
  }
};

// Search from root
findPnpmLocations(rootDir);

// Also check fixtures directory
const fixturesDir = path.join(rootDir, 'fixtures');
if (fs.existsSync(fixturesDir)) {
  findPnpmLocations(fixturesDir);
}

// Also check root node_modules/.pnpm directly
const rootPnpmDir = path.join(rootDir, 'node_modules', '.pnpm');
if (fs.existsSync(rootPnpmDir)) {
  findPnpmLocations(rootPnpmDir);

  // Also directly check for fob-native-build packages
  try {
    const entries = fs.readdirSync(rootPnpmDir, { withFileTypes: true });
    for (const entry of entries) {
      if (entry.isDirectory() && entry.name.startsWith('fob-native-build@')) {
        const packagePath = path.join(rootPnpmDir, entry.name, 'node_modules', 'fob-native-build');
        if (fs.existsSync(packagePath)) {
          targets.push(packagePath);
        }
      }
    }
  } catch (err) {
    // Ignore errors
  }
}

// Remove duplicates
const uniqueTargets = [...new Set(targets.map((t) => path.resolve(t)))];

if (uniqueTargets.length === 0) {
  console.warn('Warning: No node_modules locations found. Binary may not be synced.');
  console.warn('This is normal if dependencies are not installed yet.');
  process.exit(0);
}

// Copy binary to all targets
let successCount = 0;
let failCount = 0;

for (const targetDir of uniqueTargets) {
  const targetPath = path.join(targetDir, binaryName);

  try {
    // Ensure target directory exists
    fs.mkdirSync(targetDir, { recursive: true });

    // Copy the file
    fs.copyFileSync(sourcePath, targetPath);
    console.log(`✓ Synced to ${path.relative(rootDir, targetPath)}`);
    successCount++;
  } catch (err) {
    console.error(`✗ Failed to sync to ${path.relative(rootDir, targetPath)}: ${err.message}`);
    failCount++;
  }
}

console.log(`\nSync complete: ${successCount} succeeded, ${failCount} failed`);

if (failCount > 0) {
  process.exit(1);
}
