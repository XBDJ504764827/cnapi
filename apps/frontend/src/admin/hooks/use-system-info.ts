import { useQuery } from '@tanstack/react-query';

import { systemInfoQuery } from '@/admin/services/system-service';

/** Version, environment, start time and uptime of the API. */
export function useSystemInfo() {
  return useQuery(systemInfoQuery());
}
