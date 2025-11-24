# Publishing Guide for @fox-uni/fob

This guide explains how to publish the `@fox-uni/fob` package to npm.

## Prerequisites

1. **npm Account**: You must have access to the `@fox-uni` organization on npm
   - Visit: https://www.npmjs.com/settings/fox-uni/members

2. **npm Token**: Generate an automation token
   - Go to: https://www.npmjs.com/settings/fox-uni/tokens
   - Click "Generate New Token" → Choose "Automation" type
   - Copy the token (it will only be shown once!)

3. **GitHub Secrets**: Add the npm token to your GitHub repository
   - Go to: Repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `NPM_TOKEN`
   - Value: Paste your npm automation token

## Publishing Workflow

### 1. Local Testing (Optional but Recommended)

Test the build locally before publishing:

```bash
# Build for your current platform
cd crates/fob-native
pnpm exec napi build --platform --release

# Run tests
pnpm test

# Test the generated artifacts
node -e "const { Fob } = require('./index.js'); console.log('Loaded successfully!');"
```

### 2. Version Bump

Update the version in `crates/fob-native/package.json`:

```bash
cd crates/fob-native

# For patch releases (0.1.0 → 0.1.1)
npm version patch

# For minor releases (0.1.0 → 0.2.0)
npm version minor

# For major releases (0.1.0 → 1.0.0)
npm version major

# Or set a specific version
npm version 0.1.0
```

This will:
- Update `package.json`
- Create a git commit
- Create a git tag

### 3. Push to GitHub

```bash
# Push the commit and tags
git push origin main --tags
```

### 4. Automated CI/CD

GitHub Actions will automatically:

1. **Build** for all platforms:
   - macOS (x64, ARM64)
   - Linux (x64 glibc, x64 musl, ARM64 glibc, ARM64 musl)
   - Windows (x64)

2. **Test** on multiple Node.js versions:
   - Node 18
   - Node 20

3. **Publish** if the commit message is a version number:
   - Main package: `@fox-uni/fob`
   - Platform packages: `@fox-uni/fob-{platform}`

### 5. Verify Publication

After the CI completes:

```bash
# Check on npm
open https://www.npmjs.com/package/@fox-uni/fob

# Test installation
npm install @fox-uni/fob
# or
pnpm add @fox-uni/fob
```

## Package Structure

The published package structure:

```
@fox-uni/fob                          # Main package
├── @fox-uni/fob-darwin-arm64         # macOS ARM64
├── @fox-uni/fob-darwin-x64           # macOS x64
├── @fox-uni/fob-linux-arm64-gnu      # Linux ARM64 (glibc)
├── @fox-uni/fob-linux-arm64-musl     # Linux ARM64 (musl)
├── @fox-uni/fob-linux-x64-gnu        # Linux x64 (glibc)
├── @fox-uni/fob-linux-x64-musl       # Linux x64 (musl)
└── @fox-uni/fob-win32-x64-msvc       # Windows x64
```

## Manual Publishing (Emergency Only)

If you need to publish manually (not recommended):

```bash
cd crates/fob-native

# Build for all platforms (requires access to all platforms)
pnpm exec napi build --platform --release --target x86_64-apple-darwin
pnpm exec napi build --platform --release --target aarch64-apple-darwin
# ... repeat for all platforms

# Prepare artifacts
pnpm exec napi artifacts

# Publish
npm publish --access public
```

## Troubleshooting

### Build Fails on CI

1. Check the GitHub Actions logs
2. Ensure all Rust dependencies are correctly specified
3. Verify the Cargo.lock is up to date

### npm Token Expired

1. Generate a new token: https://www.npmjs.com/settings/fox-uni/tokens
2. Update the `NPM_TOKEN` secret in GitHub

### Wrong Version Published

You can unpublish within 72 hours:

```bash
npm unpublish @fox-uni/fob@<version>
```

Then fix and republish with the correct version.

### Platform Package Missing

If a specific platform package is missing:

1. Check the GitHub Actions artifacts
2. Verify the build succeeded for that platform
3. Re-run the failed job if needed

## Release Checklist

Before each release:

- [ ] All tests pass locally (`pnpm test`)
- [ ] Update CHANGELOG.md with changes
- [ ] Version bump is correct (`npm version`)
- [ ] Tag is pushed (`git push --tags`)
- [ ] CI builds succeed
- [ ] Manual test installation works

## Version Policy

- **Patch** (0.1.x): Bug fixes, minor improvements
- **Minor** (0.x.0): New features, non-breaking changes
- **Major** (x.0.0): Breaking API changes

## Support

For issues with publishing:
- GitHub Issues: https://github.com/fox-uni/fob/issues
- npm Organization: https://www.npmjs.com/settings/fox-uni/members

