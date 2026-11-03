import path from 'node:path';
import { fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const apiTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:8080';
const adminTarget = process.env.ADMIN_DEV_ORIGIN ?? 'http://127.0.0.1:5174';

/** Public portal: same-origin API via dev proxy / production static server proxy. */
export default defineConfig({
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
    proxy: {
      '/admin': { target: adminTarget, changeOrigin: true, ws: true },
      '/api': { target: apiTarget, changeOrigin: true },
      '/health': { target: apiTarget, changeOrigin: true },
      '/swagger-ui': { target: apiTarget, changeOrigin: true },
      '/api-docs': { target: apiTarget, changeOrigin: true },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
  },
});
