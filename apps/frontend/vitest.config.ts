import path from 'node:path';
import { fileURLToPath } from 'node:url';

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

const rootDir = path.dirname(fileURLToPath(import.meta.url));

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
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
