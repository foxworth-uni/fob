/**
 * Entry point with dynamic imports for code splitting
 */

export async function loadFeature(name) {
  if (name === 'feature-a') {
    const module = await import('./feature-a.js');
    return module.featureA();
  }
  if (name === 'feature-b') {
    const module = await import('./feature-b.js');
    return module.featureB();
  }
  throw new Error(`Unknown feature: ${name}`);
}

export const staticExport = 'main-module';
