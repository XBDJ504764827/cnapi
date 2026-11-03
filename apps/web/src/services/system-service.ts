import { queryOptions } from '@tanstack/react-query';

import { api } from '@/lib/api';

/**
 * System endpoints. Pages use the hooks in `src/hooks`, never `api` directly.
 */
export const systemInfoQuery = () =>
  queryOptions({
    queryKey: ['system', 'info'],
    queryFn: () => api.system.getInfo(),
    staleTime: 30_000,
    refetchInterval: 60_000,
  });

export const systemStatusQuery = () =>
  queryOptions({
    queryKey: ['system', 'status'],
    queryFn: () => api.system.getStatus(),
    staleTime: 5_000,
    refetchInterval: 15_000,
  });

export const systemEventsQuery = (limit = 10) =>
  queryOptions({
    queryKey: ['system', 'events', limit],
    queryFn: () => api.system.getRecentEvents(limit),
    staleTime: 15_000,
    refetchInterval: 60_000,
  });

export const readinessQuery = () =>
  queryOptions({
    queryKey: ['health', 'ready'],
    queryFn: () => api.health.getReadiness(),
    staleTime: 5_000,
    refetchInterval: 15_000,
  });
