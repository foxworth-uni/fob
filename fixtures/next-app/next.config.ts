import type { NextConfig } from 'next';

const nextConfig: NextConfig = {
  // Externalize native modules so they're not bundled by Turbopack
  serverExternalPackages: ['@fox-uni/fob'],
};

export default nextConfig;
