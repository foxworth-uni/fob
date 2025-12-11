import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // Externalize native modules so they're not bundled by webpack
  serverExternalPackages: ['@fox-uni/fob'],

  // Configure webpack to handle native modules
  webpack: (config, { isServer }) => {
    if (isServer) {
      // Don't bundle native node modules
      config.externals = config.externals || [];
      config.externals.push({
        '@fox-uni/fob': 'commonjs @fox-uni/fob',
      });
    }
    return config;
  },
};

export default nextConfig;
