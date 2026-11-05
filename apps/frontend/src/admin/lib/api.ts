import { createApiClient } from '@cngokz/api-client';

import { env } from './env';

/**
 * Single API client instance. Pages never build URLs or `fetch` calls by
 * themselves, they go through hooks -> services -> this client.
 */
export const api = createApiClient({ baseUrl: env.VITE_API_BASE_URL });
