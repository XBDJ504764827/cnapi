import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render } from '@testing-library/react';
import { createElement, type ReactElement } from 'react';
import { MemoryRouter, Route, Routes } from 'react-router';

import { TooltipProvider } from '@/components/ui/tooltip';

interface RenderPageOptions {
  path?: string;
  entry?: string;
}

/** Renders a page with a fresh QueryClient, router and tooltip provider. */
export function renderPage(ui: ReactElement, { path = '/', entry = '/' }: RenderPageOptions = {}) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
  });

  const routed = path === '/' ? ui : createElement(Routes, null, createElement(Route, { path, element: ui }));

  return render(
    createElement(
      QueryClientProvider,
      { client },
      createElement(TooltipProvider, null, createElement(MemoryRouter, { initialEntries: [entry] }, routed)),
    ),
  );
}
