import { defineConfig } from 'vite';
import { resolve } from 'path';
import dts from 'vite-plugin-dts';

export default defineConfig({
  build: {
    lib: {
      entry: resolve(__dirname, 'src/index.ts'),
      name: 'MirrorEditor',
      formats: ['es'],
      fileName: 'index',
    },
    rollupOptions: {
      external: ['codemirror', /^@codemirror\//],
      output: {
        preserveModules: false,
        globals: {
          codemirror: 'CodeMirror',
        },
        assetFileNames: (assetInfo) => {
          if (assetInfo.name && assetInfo.name.endsWith('.css')) {
            return 'style.css';
          }
          return assetInfo.name || 'assets/[name]-[hash][extname]';
        },
      },
    },
    sourcemap: true,
    minify: false,
    cssCodeSplit: false,
  },
  plugins: [
    dts({
      insertTypesEntry: true,
      rollupTypes: true,
    }),
  ],
});
