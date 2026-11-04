import { keepPreviousData, queryOptions } from '@tanstack/react-query';

import { api } from '@/lib/api';

const stale = { staleTime: 30_000 };

export const pointsQueries = {
  player: (steamid: string) =>
    queryOptions({
      queryKey: ['points', 'player', steamid],
      queryFn: () => api.points.getPlayerPoints(steamid),
      ...stale,
    }),
  playerFilters: (steamid: string, query: Record<string, string | number | undefined>) =>
    queryOptions({
      queryKey: ['points', 'player', steamid, 'filters', query],
      queryFn: () => api.points.getPlayerFilterPoints(steamid, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  mode: (mode: string, page = 1, pageSize = 20) =>
    queryOptions({
      queryKey: ['points', 'mode', mode, page, pageSize],
      queryFn: () => api.points.getModeLeaderboard(mode, { page, page_size: pageSize }),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  filter: (query: Record<string, string | number>) =>
    queryOptions({
      queryKey: ['points', 'filter', query],
      queryFn: () => api.points.getFilterLeaderboard(query as never),
      ...stale,
    }),
  status: () =>
    queryOptions({
      queryKey: ['points', 'status'],
      queryFn: () => api.points.getStatus(),
      staleTime: 15_000,
      refetchInterval: 30_000,
    }),
  tiers: (mode: string | undefined, page = 1, pageSize = 20) =>
    queryOptions({
      queryKey: ['points', 'tiers', mode, page, pageSize],
      queryFn: () => api.points.getAdminTiers({ mode, page, page_size: pageSize }),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  runs: (page = 1, pageSize = 20) =>
    queryOptions({
      queryKey: ['points', 'runs', page, pageSize],
      queryFn: () => api.points.getRecalcRuns({ page, page_size: pageSize }),
      placeholderData: keepPreviousData,
      ...stale,
    }),
};
