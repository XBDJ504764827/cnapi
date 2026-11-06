import { keepPreviousData, queryOptions } from '@tanstack/react-query';
import type {
  PublicLeaderboardQuery,
  PublicMapListQuery,
  PublicPlayerDetailQuery,
  PublicPlayerListQuery,
  PublicPlayerMapsQuery,
  PublicPlayerRecordsQuery,
  PublicRecordQuery,
} from '@cngokz/api-client';

import { api } from '@/admin/lib/api';

/**
 * 管理端只读数据。
 *
 * 这些页面（玩家 / 地图 / 成绩 / 排行榜）读取的是公开只读接口——数据完全一致，
 * 后端已有 Redis 缓存，管理端不需要再复制一套查询。写操作（审核、封禁、改名等）
 * 将来才会走 `/api/v1/admin/*`。
 */
const stale = { staleTime: 30_000 };

export const directoryQueries = {
  players: (query: PublicPlayerListQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'players', query],
      queryFn: () => api.public.getPlayers(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),

  player: (steamid: string, query: PublicPlayerDetailQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'player', steamid, query],
      queryFn: () => api.public.getPlayer(steamid, query),
      ...stale,
    }),

  playerRecords: (steamid: string, query: PublicPlayerRecordsQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'player', steamid, 'records', query],
      queryFn: () => api.public.getPlayerRecords(steamid, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),

  playerMaps: (steamid: string, query: PublicPlayerMapsQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'player', steamid, 'maps', query],
      queryFn: () => api.public.getPlayerMaps(steamid, query),
      placeholderData: keepPreviousData,
      ...stale,
    }),

  maps: (query: PublicMapListQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'maps', query],
      queryFn: () => api.public.getMaps(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),

  records: (query: PublicRecordQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'records', query],
      queryFn: () => api.public.getRecords(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),

  leaderboard: (query: PublicLeaderboardQuery) =>
    queryOptions({
      queryKey: ['admin', 'directory', 'leaderboard', query],
      queryFn: () => api.public.getLeaderboard(query),
      placeholderData: keepPreviousData,
      ...stale,
    }),
};
