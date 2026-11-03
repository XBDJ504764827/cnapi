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
  };
}

export type ApiClient = ReturnType<typeof createApiClient>;
