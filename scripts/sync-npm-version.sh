#!/usr/bin/env bash
set -euo pipefail

# Called by cargo-release pre-release-hook
# Environment variables from cargo-release:
#   NEW_VERSION - the version being released
#   DRY_RUN - "true" if --dry-run

VERSION="${NEW_VERSION}"
NPM_PKG="crates/fob-native/package.json"

if [ "$DRY_RUN" = "true" ]; then
    echo "[dry-run] Would update $NPM_PKG to version $VERSION"
    exit 0
fi

echo "Syncing npm package version to $VERSION..."

# Update main package.json version
# Using node for reliable JSON manipulation
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('$NPM_PKG', 'utf8'));
pkg.version = '$VERSION';
fs.writeFileSync('$NPM_PKG', JSON.stringify(pkg, null, 2) + '\n');
"

echo "Updated $NPM_PKG to version $VERSION"

# Stage the file so cargo-release includes it in the commit
git add "$NPM_PKG"
