import { QueryClient } from '@tanstack/react-query';

/** Shared TanStack Query client. */
export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      refetchOnWindowFocus: false,
      staleTime: 10_000,
    },
  },
});
