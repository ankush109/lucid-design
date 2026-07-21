import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { viteSingleFile } from 'vite-plugin-singlefile';

// Single-file bundle: everything inlined into dist/index.html so wry can
// embed it via `include_str!`. Assets are base64'd; no external network.
export default defineConfig({
  plugins: [react(), viteSingleFile()],
  build: {
    target: 'safari14',
    cssCodeSplit: false,
    assetsInlineLimit: 100_000_000,
    chunkSizeWarningLimit: 4096,
    rollupOptions: {
      output: { inlineDynamicImports: true },
    },
  },
  server: { port: 5173 },
});
