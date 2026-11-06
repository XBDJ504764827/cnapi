import { queryOptions, useQuery } from '@tanstack/react-query';

import { api } from '@/admin/lib/api';

/**
 * 仪表盘的业务统计。
 *
 * 数据来自公开只读接口 `/api/v1/public/statistics`（后端有 Redis 缓存，30 秒有效），
 * 因此管理端不需要另开一套统计接口。
 */
export const dashboardStatsQuery = queryOptions({
  queryKey: ['admin', 'dashboard', 'stats'],
  queryFn: () => api.public.getStatistics(),
  staleTime: 30_000,
  refetchInterval: 60_000,
});

export function useDashboardStats() {
  return useQuery(dashboardStatsQuery);
}
