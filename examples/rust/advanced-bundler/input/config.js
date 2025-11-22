// Application configuration
// Shared across multiple entry points

export const APP_CONFIG = {
  name: 'Advanced Bundler Example',
  version: '2.0.0',
  author: 'Fob Team',
  features: [
    'Multiple entry points',
    'Code splitting',
    'Path aliases',
    'Multiple formats',
    'External dependencies',
  ],
};

export const BUILD_CONFIG = {
  minify: true,
  sourceMaps: true,
  target: 'es2020',
};
