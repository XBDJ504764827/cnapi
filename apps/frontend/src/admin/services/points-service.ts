import { queryOptions } from '@tanstack/react-query';

import { api } from '@/admin/lib/api';

/** Points administration reads. The maintenance token is injected per request. */
function authHeaders(): Record<string, string> {
  const token = import.meta.env.VITE_MAINTENANCE_TOKEN ?? '';
  return token ? { 'X-Maintenance-Token': token } : {};
}

export const pointsQueries = {
  status: () =>
    queryOptions({
      queryKey: ['points', 'status'],
      queryFn: () => api.points.getStatus({ headers: authHeaders() }),
      refetchInterval: 15_000,
    }),
  tiers: (mode: string | undefined, page = 1, pageSize = 20) =>
    queryOptions({
      queryKey: ['points', 'tiers', mode, page, pageSize],
      queryFn: () => api.points.getAdminTiers({ mode, page, page_size: pageSize }, { headers: authHeaders() }),
    }),
  runs: (page = 1, pageSize = 20) =>
    queryOptions({
      queryKey: ['points', 'runs', page, pageSize],
      queryFn: () => api.points.getRecalcRuns({ page, page_size: pageSize }, { headers: authHeaders() }),
      refetchInterval: 15_000,
    }),
  player: (steamid: string) =>
    queryOptions({
      queryKey: ['points', 'player', steamid, 'summary'],
      queryFn: () => api.points.getPlayerPoints(steamid),
    }),
  playerFilters: (steamid: string, query: Record<string, string | number | undefined>) =>
    queryOptions({
      queryKey: ['points', 'player', steamid, 'filters', query],
      queryFn: () => api.points.getPlayerFilterPoints(steamid, query),
    }),
  modeLeaderboard: (mode: string, page = 1, pageSize = 10) =>
    queryOptions({
      queryKey: ['points', 'leaderboard', mode, page, pageSize],
      queryFn: () => api.points.getModeLeaderboard(mode, { page, page_size: pageSize }),
    }),
};
