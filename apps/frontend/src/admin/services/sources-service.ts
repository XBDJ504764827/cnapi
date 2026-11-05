import { env } from '@/admin/lib/env';

function token(): string | undefined {
  const value = import.meta.env.VITE_MAINTENANCE_TOKEN as string | undefined;
  return value?.trim() ? value.trim() : undefined;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const value = token();
  const response = await fetch(`${env.VITE_API_BASE_URL}${path}`, {
    ...init,
    headers: {
      ...(init?.body ? { 'Content-Type': 'application/json' } : {}),
      ...(value ? { 'X-Maintenance-Token': value } : {}),
      ...(init?.headers as Record<string, string> | undefined),
    },
  });
  const body = (await response.json()) as { data?: T; message?: string };
  if (!response.ok) throw new Error(body.message ?? `请求失败（HTTP ${response.status}）`);
  return body.data as T;
}

export interface SourceSummary {
  id: number;
  code: string;
  source_type: string;
  display_name: string;
  is_enabled: boolean;
  priority: number;
  status: string;
  last_attempt_at: string | null;
  last_success_at: string | null;
  last_failure_at: string | null;
  last_error: string | null;
  consecutive_failures: number;
  cursor: Record<string, unknown>;
  config: Record<string, unknown>;
  raw_records: number;
  canonical_links: number;
  latest_run_status: string | null;
  latest_run_at: string | null;
}

export interface SourceRun {
  id: number;
  source_id: number;
  run_type: string;
  status: string;
  started_at: string;
  completed_at: string | null;
  stats: Record<string, unknown>;
  error: string | null;
}

export interface SourceRunsResponse {
  items: SourceRun[];
  meta: { page: number; page_size: number; total: number; total_pages: number };
}

export const sourceAdmin = {
  list: () => request<{ items: SourceSummary[] }>('/api/v1/admin/sources'),
  setEnabled: (code: string, enabled: boolean) =>
    request<SourceSummary>(`/api/v1/admin/sources/${code}`, {
      method: 'PATCH',
      body: JSON.stringify({ is_enabled: enabled }),
    }),
  sync: (code: string) =>
    request<{ source: string; status: string; message: string; stats: Record<string, unknown> }>(
      `/api/v1/admin/sources/${code}/sync`,
      { method: 'POST' },
    ),
  runs: (code: string, page = 1, pageSize = 20) =>
    request<SourceRunsResponse>(`/api/v1/admin/sources/${code}/runs?page=${page}&page_size=${pageSize}`),
};

export const sourceKeys = {
  all: ['admin', 'sources'] as const,
  list: () => [...sourceKeys.all, 'list'] as const,
  runs: (code: string) => [...sourceKeys.all, 'runs', code] as const,
};
