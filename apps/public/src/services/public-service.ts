import { keepPreviousData, queryOptions } from '@tanstack/react-query';
import type {
  PublicActivePlayersQuery,
  PublicCompareQuery,
  PublicLeaderboardQuery,
  PublicMapDetailQuery,
  PublicMapLeaderboardQuery,
  PublicMapListQuery,
  PublicPlayerDetailQuery,
  PublicPlayerListQuery,
  PublicPlayerMapsQuery,
  PublicPlayerRecordsQuery,
  PublicRecordQuery,
  PublicRecentRecordsQuery,
  PublicRecentWorldRecordsQuery,
  PublicServerQuery,
} from '@cngokz/api-client';

import { api } from '@/lib/api';

const stale = { staleTime: 30_000 };

export const publicQueries = {
  statistics: () =>
    queryOptions({
      queryKey: ['public', 'statistics'],
      queryFn: () => api.public.getStatistics(),
      staleTime: 30_000,
      refetchInterval: 60_000,
    }),
  modes: () =>
    queryOptions({
      queryKey: ['public', 'modes'],
      queryFn: () => api.public.getModes(),
      staleTime: 5 * 60_000,
    }),
  leaderboard: (query: PublicLeaderboardQuery) =>
    queryOptions({
      queryKey: ['public', 'leaderboard', query],
      queryFn: () => api.public.getLeaderboard(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  players: (query: PublicPlayerListQuery) =>
    queryOptions({
      queryKey: ['public', 'players', query],
      queryFn: () => api.public.getPlayers(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  activePlayers: (query: PublicActivePlayersQuery) =>
    queryOptions({
      queryKey: ['public', 'players', 'active', query],
      queryFn: () => api.public.getActivePlayers(query),
      staleTime: 60_000,
    }),
  player: (steamid: string, query: PublicPlayerDetailQuery) =>
    queryOptions({
      queryKey: ['public', 'player', steamid, query],
      queryFn: () => api.public.getPlayer(steamid, query),
      ...stale,
    }),
  playerRecords: (steamid: string, query: PublicPlayerRecordsQuery) =>
    queryOptions({
      queryKey: ['public', 'player', steamid, 'records', query],
      queryFn: () => api.public.getPlayerRecords(steamid, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  playerMaps: (steamid: string, query: PublicPlayerMapsQuery) =>
    queryOptions({
      queryKey: ['public', 'player', steamid, 'maps', query],
      queryFn: () => api.public.getPlayerMaps(steamid, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  compare: (query: PublicCompareQuery) =>
    queryOptions({
      queryKey: ['public', 'compare', query],
      queryFn: () => api.public.comparePlayers(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  maps: (query: PublicMapListQuery) =>
    queryOptions({
      queryKey: ['public', 'maps', query],
      queryFn: () => api.public.getMaps(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  map: (slug: string, query: PublicMapDetailQuery) =>
    queryOptions({
      queryKey: ['public', 'map', slug, query],
      queryFn: () => api.public.getMap(slug, query),
      ...stale,
    }),
  mapLeaderboard: (slug: string, query: PublicMapLeaderboardQuery) =>
    queryOptions({
      queryKey: ['public', 'map', slug, 'leaderboard', query],
      queryFn: () => api.public.getMapLeaderboard(slug, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  servers: (query: PublicServerQuery) =>
    queryOptions({
      queryKey: ['public', 'servers', query],
      queryFn: () => api.public.getServers(query),
      placeholderData: keepPreviousData,
      staleTime: 15_000,
      refetchInterval: 30_000,
    }),
  records: (query: PublicRecordQuery) =>
    queryOptions({
      queryKey: ['public', 'records', query],
      queryFn: () => api.public.getRecords(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
  recentRecords: (query: PublicRecentRecordsQuery) =>
    queryOptions({
      queryKey: ['public', 'records', 'recent', query],
      queryFn: () => api.public.getRecentRecords(query),
      staleTime: 15_000,
    }),
  worldRecords: (query: PublicRecentWorldRecordsQuery) =>
    queryOptions({
      queryKey: ['public', 'records', 'world', query],
      queryFn: () => api.public.getRecentWorldRecords(query),
      staleTime: 30_000,
    }),
  record: (id: number) =>
    queryOptions({
      queryKey: ['public', 'record', id],
      queryFn: () => api.public.getRecord(id),
      ...stale,
    }),
  search: (q: string) =>
    queryOptions({
      queryKey: ['public', 'search', q],
      queryFn: () => api.public.search(q),
      enabled: q.trim().length >= 2,
      staleTime: 30_000,
    }),
};
