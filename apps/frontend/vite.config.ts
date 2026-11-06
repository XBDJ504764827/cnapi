import path from 'node:path';
import { fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const apiTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:8080';
const proxyTo = Object.fromEntries(
  ['/api', '/health', '/swagger-ui', '/api-docs'].map((prefix) => [prefix, { target: apiTarget, changeOrigin: true }]),
);

/** Single Vite app: public portal at `/`, admin UI at `/admin/`. */
export default defineConfig({
  base: '/',
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      '@': path.resolve(rootDir, 'src'),
      '@cngokz/shared-types': path.resolve(rootDir, '../../packages/shared-types/src'),
      '@cngokz/api-client': path.resolve(rootDir, '../../packages/api-client/src'),
      '@cngokz/ui/styles.css': path.resolve(rootDir, '../../packages/ui/src/design-system.css'),
    },
  },
  server: {
    host: true,
    port: Number(process.env.PORTAL_PORT ?? 5173),
    strictPort: true,
    proxy: proxyTo,
  },
  preview: {
    host: true,
    port: Number(process.env.PORTAL_PORT ?? 5173),
    proxy: proxyTo,
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
  },
});
