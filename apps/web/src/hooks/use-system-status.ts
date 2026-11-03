import { useQuery } from '@tanstack/react-query';

import { systemStatusQuery } from '@/services/system-service';

/** Live dependency status, polled every 15s. */
export function useSystemStatus() {
  return useQuery(systemStatusQuery());
}
