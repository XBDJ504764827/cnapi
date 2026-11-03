import path from 'node:path';
import { fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const apiTarget = process.env.VITE_API_PROXY_TARGET ?? 'http://localhost:8080';

/**
 * The web app talks to the API through relative URLs by default
 * (`VITE_API_BASE_URL=""`), which keeps Docker, preview and dev server
 * deployments free of hardcoded hosts:
 * - `vite dev` proxies `/api`, `/health`, `/swagger-ui`, `/api-docs` to the API
 * - the production static server (apps/web/server/static-server.mjs) proxies the
 *   same prefixes to `API_UPSTREAM`
 */
export default defineConfig({
  base: '/admin/',
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
    port: Number(process.env.ADMIN_PORT ?? 5174),
    strictPort: true,
    proxy: proxyTo(apiTarget),
  },
  preview: {
    host: true,
    port: Number(process.env.ADMIN_PORT ?? 5174),
    proxy: proxyTo(apiTarget),
  },
  build: {
    outDir: 'dist',
    sourcemap: false,
  },
});

function proxyTo(target: string) {
  const prefixes = ['/api', '/health', '/swagger-ui', '/api-docs'];
  return Object.fromEntries(prefixes.map((prefix) => [prefix, { target, changeOrigin: true }])) as Record<
    string,
    { target: string; changeOrigin: boolean }
  >;
}
