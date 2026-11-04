import createClient from 'openapi-fetch';

import type { paths } from '@cngokz/shared-types';

import { request } from './request';

export interface ApiClientOptions {
  /** Base URL without a trailing slash. Empty string means same origin. */
  baseUrl: string;
  /** Optional fetch implementation (used by tests). */
  fetch?: typeof globalThis.fetch;
}

/**
 * Typed API client. All HTTP calls of the platform go through this factory so
 * endpoints, request parameters and response envelopes stay in sync with the
 * backend OpenAPI document.
 */
export function createApiClient({ baseUrl, fetch }: ApiClientOptions) {
  const http = createClient<paths>({
    baseUrl,
    ...(fetch ? { fetch } : {}),
  });

  return {
    http,
    health: {
      getHealth: () => request(http.GET('/health')),
      getLiveness: () => request(http.GET('/health/live')),
      getReadiness: () => request(http.GET('/health/ready')),
    },
    system: {
      getInfo: () => request(http.GET('/api/v1/system/info')),
      getStatus: () => request(http.GET('/api/v1/system/status')),
      getRecentEvents: (limit?: number) =>
        request(
          http.GET('/api/v1/system/events', {
            params: { query: limit === undefined ? {} : { limit } },
          }),
        ),
    },
    points: {
      getPlayerPoints: (steamid: string) =>
        request(http.GET('/api/v1/players/{steamid}/points', { params: { path: { steamid } } })),
      getPlayerFilterPoints: (steamid: string, query?: PublicPlayerFilterPointsQuery) =>
        request(
          http.GET('/api/v1/players/{steamid}/points/filters', {
            params: { path: { steamid }, query: query ?? {} },
          }),
        ),
      getModeLeaderboard: (mode: string, query?: PublicPointsLeaderboardQuery) =>
        request(
          http.GET('/api/v1/leaderboards/points/{mode}', {
            params: { path: { mode }, query: query ?? {} },
          }),
        ),
      getFilterLeaderboard: (query: PublicFilterLeaderboardQuery) =>
        request(http.GET('/api/v1/leaderboards/filters', { params: { query } })),
      getStatus: (init?: { headers?: Record<string, string> }) =>
        request(http.GET('/api/v1/admin/points/status', init ?? {})),
      getAdminTiers: (query?: PublicAdminTiersQuery, init?: { headers?: Record<string, string> }) =>
        request(http.GET('/api/v1/admin/points/tiers', { params: { query: query ?? {} }, ...(init ?? {}) })),
      getRecalcRuns: (query?: PublicRecalcRunsQuery, init?: { headers?: Record<string, string> }) =>
        request(http.GET('/api/v1/admin/points/recalc-runs', { params: { query: query ?? {} }, ...(init ?? {}) })),
    },
    public: {
      getStatistics: () => request(http.GET('/api/v1/public/statistics')),
      getModes: () => request(http.GET('/api/v1/public/modes')),
      getLeaderboard: (query?: PublicLeaderboardQuery) =>
        request(http.GET('/api/v1/public/leaderboards', { params: { query: query ?? {} } })),
      getRecords: (query?: PublicRecordQuery) =>
        request(http.GET('/api/v1/public/records', { params: { query: query ?? {} } })),
      getRecentRecords: (query?: PublicRecentRecordsQuery) =>
        request(http.GET('/api/v1/public/records/recent', { params: { query: query ?? {} } })),
      getRecentWorldRecords: (query?: PublicRecentWorldRecordsQuery) =>
        request(http.GET('/api/v1/public/records/world-records', { params: { query: query ?? {} } })),
      getRecord: (id: number) => request(http.GET('/api/v1/public/records/{id}', { params: { path: { id } } })),
      getPlayers: (query?: PublicPlayerListQuery) =>
        request(http.GET('/api/v1/public/players', { params: { query: query ?? {} } })),
      getActivePlayers: (query?: PublicActivePlayersQuery) =>
        request(http.GET('/api/v1/public/players/active', { params: { query: query ?? {} } })),
      getOnlinePlayers: (query?: PublicOnlinePlayersQuery) =>
        request(http.GET('/api/v1/public/players/online', { params: { query: query ?? {} } })),
      comparePlayers: (query: PublicCompareQuery) =>
        request(http.GET('/api/v1/public/players/compare', { params: { query } })),
      getPlayer: (steamid: string, query?: PublicPlayerDetailQuery) =>
        request(http.GET('/api/v1/public/players/{steamid}', { params: { path: { steamid }, query: query ?? {} } })),
      getPlayerRecords: (steamid: string, query?: PublicPlayerRecordsQuery) =>
        request(
          http.GET('/api/v1/public/players/{steamid}/records', {
            params: { path: { steamid }, query: query ?? {} },
          }),
        ),
      getPlayerMaps: (steamid: string, query?: PublicPlayerMapsQuery) =>
        request(
          http.GET('/api/v1/public/players/{steamid}/maps', {
            params: { path: { steamid }, query: query ?? {} },
          }),
        ),
      getMaps: (query?: PublicMapListQuery) =>
        request(http.GET('/api/v1/public/maps', { params: { query: query ?? {} } })),
      getMap: (slug: string, query?: PublicMapDetailQuery) =>
        request(http.GET('/api/v1/public/maps/{slug}', { params: { path: { slug }, query: query ?? {} } })),
      getMapLeaderboard: (slug: string, query?: PublicMapLeaderboardQuery) =>
        request(
          http.GET('/api/v1/public/maps/{slug}/leaderboard', {
            params: { path: { slug }, query: query ?? {} },
          }),
        ),
      getMapStatistics: (slug: string, query?: PublicMapDetailQuery) =>
        request(
          http.GET('/api/v1/public/maps/{slug}/statistics', {
            params: { path: { slug }, query: query ?? {} },
          }),
        ),
      getServers: (query?: PublicServerQuery) =>
        request(http.GET('/api/v1/public/servers', { params: { query: query ?? {} } })),
      getServer: (id: number) => request(http.GET('/api/v1/public/servers/{id}', { params: { path: { id } } })),
      search: (q: string) => request(http.GET('/api/v1/public/search', { params: { query: { q } } })),
    },
  };
}

type QueryOf<P extends keyof paths, M extends keyof paths[P]> = paths[P][M] extends {
  parameters: { query?: infer Q };
}
  ? Q
  : never;

export type PublicLeaderboardQuery = NonNullable<QueryOf<'/api/v1/public/leaderboards', 'get'>>;
export type PublicRecordQuery = NonNullable<QueryOf<'/api/v1/public/records', 'get'>>;
export type PublicRecentRecordsQuery = NonNullable<QueryOf<'/api/v1/public/records/recent', 'get'>>;
export type PublicRecentWorldRecordsQuery = NonNullable<QueryOf<'/api/v1/public/records/world-records', 'get'>>;
export type PublicPlayerListQuery = NonNullable<QueryOf<'/api/v1/public/players', 'get'>>;
export type PublicActivePlayersQuery = NonNullable<QueryOf<'/api/v1/public/players/active', 'get'>>;
export type PublicOnlinePlayersQuery = NonNullable<QueryOf<'/api/v1/public/players/online', 'get'>>;
export type PublicCompareQuery = NonNullable<QueryOf<'/api/v1/public/players/compare', 'get'>>;
export type PublicPlayerDetailQuery = NonNullable<QueryOf<'/api/v1/public/players/{steamid}', 'get'>>;
export type PublicPlayerMapsQuery = NonNullable<QueryOf<'/api/v1/public/players/{steamid}/maps', 'get'>>;
export type PublicPlayerRecordsQuery = NonNullable<QueryOf<'/api/v1/public/players/{steamid}/records', 'get'>>;
export type PublicMapListQuery = NonNullable<QueryOf<'/api/v1/public/maps', 'get'>>;
export type PublicMapDetailQuery = NonNullable<QueryOf<'/api/v1/public/maps/{slug}', 'get'>>;
export type PublicMapLeaderboardQuery = NonNullable<QueryOf<'/api/v1/public/maps/{slug}/leaderboard', 'get'>>;
export type PublicServerQuery = NonNullable<QueryOf<'/api/v1/public/servers', 'get'>>;
export type PublicPlayerFilterPointsQuery = NonNullable<QueryOf<'/api/v1/players/{steamid}/points/filters', 'get'>>;
export type PublicPointsLeaderboardQuery = NonNullable<QueryOf<'/api/v1/leaderboards/points/{mode}', 'get'>>;
export type PublicFilterLeaderboardQuery = QueryOf<'/api/v1/leaderboards/filters', 'get'>;
export type PublicAdminTiersQuery = NonNullable<QueryOf<'/api/v1/admin/points/tiers', 'get'>>;
export type PublicRecalcRunsQuery = NonNullable<QueryOf<'/api/v1/admin/points/recalc-runs', 'get'>>;

export type ApiClient = ReturnType<typeof createApiClient>;
