import { createApiClient } from '@cngokz/api-client';

import { env } from './env';

export const api = createApiClient({ baseUrl: env.VITE_API_BASE_URL });
