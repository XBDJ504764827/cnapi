import { QueryClientProvider } from '@tanstack/react-query';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { RouterProvider } from 'react-router';

import { ThemeProvider } from '@/components/theme/theme-provider';
import { queryClient } from '@/lib/query-client';
import { router } from '@/router';
import '@/styles/index.css';

const container = document.getElementById('root');

if (!container) {
  throw new Error('Root container #root is missing in index.html');
}

createRoot(container).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <ThemeProvider>
        <RouterProvider router={router} />
      </ThemeProvider>
    </QueryClientProvider>
  </StrictMode>,
);
