import { useQuery } from '@tanstack/react-query';

import { systemEventsQuery } from '@/admin/services/system-service';

/** Most recent system events (empty for a fresh installation). */
export function useSystemEvents(limit = 10) {
  return useQuery(systemEventsQuery(limit));
}
