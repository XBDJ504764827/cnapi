import { env } from '@/admin/lib/env';

/**
 * Phase 7 admin API access. The backend protects these routes with the Steam
 * session cookie (admin role); an anonymous browser gets 401 and the UI shows
 * the real error instead of pretending success.
 */
async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${env.VITE_API_BASE_URL}${path}`, {
    ...init,
    credentials: 'include',
    headers: {
      ...(init?.body ? { 'Content-Type': 'application/json' } : {}),
      ...(init?.headers as Record<string, string> | undefined),
    },
  });
  const body = (await response.json()) as { data?: T; message?: string };
  if (!response.ok) {
    throw new Error(body.message ?? `请求失败（HTTP ${response.status}）`);
  }
  return body.data as T;
}

export interface AdminServerSummary {
  id: number;
  name: string;
  ip: string;
  port: number;
  query_port: number | null;
  game: string;
  region: string | null;
  server_type: string;
  max_players: number;
  online_players: number;
  current_map_id: number | null;
  current_map_name: string | null;
  current_mode: string | null;
  status: string;
  is_enabled: boolean;
  is_online: boolean;
  plugin_version: string | null;
  last_heartbeat_at: string | null;
  heartbeat_age_seconds: number | null;
  api_key_prefix: string;
  api_key_created_at: string;
  api_key_revoked_at: string | null;
  created_at: string;
}

export interface AdminServerList {
  items: AdminServerSummary[];
  meta: { page: number; page_size: number; total: number; total_pages: number };
}

export interface AdminHeartbeatRow {
  id: number;
  received_at: string;
  map_name: string | null;
  mode_code: string | null;
  online_players: number;
  max_players: number;
  plugin_version: string | null;
  game_version: string | null;
}

export interface AdminPlayerRow {
  account_id: number;
  steam_id64: string;
  display_name: string;
  status: string;
  connected_at: string;
  last_seen_at: string;
}

export interface AdminRecordRow {
  id: number;
  account_id: number;
  steam_id64: string;
  display_name: string | null;
  map_name: string;
  course_stage: number;
  mode_code: string;
  time_type: string;
  time_ns: number;
  time_seconds: string;
  status: string;
  is_world_record: boolean;
  replay_status: string | null;
  created_at: string;
}

export interface AdminReplayRow {
  id: number;
  record_id: number;
  bucket: string;
  object_key: string;
  size_bytes: number;
  status: string;
  uploaded_at: string | null;
  created_at: string;
}

export interface AdminSyncLogRow {
  id: number;
  kind: string;
  status: string;
  message: string | null;
  record_id: number | null;
  created_at: string;
}

export interface AdminSyncTaskRow {
  id: number;
  record_id: number | null;
  task_type: string;
  status: string;
  attempts: number;
  last_error: string | null;
  created_at: string;
  updated_at: string;
}

export interface AdminServerDetail {
  server: AdminServerSummary;
  recent_heartbeats: AdminHeartbeatRow[];
  online_players: AdminPlayerRow[];
  recent_records: AdminRecordRow[];
  recent_replays: AdminReplayRow[];
  sync_logs: AdminSyncLogRow[];
  sync_tasks: AdminSyncTaskRow[];
}

export interface CreateServerInput {
  name: string;
  ip: string;
  port: number;
  query_port?: number;
  region?: string;
  server_type?: string;
  max_players?: number;
  owner_steam_id64: string;
  owner_display_name?: string;
}

export interface CreateServerResult {
  server: AdminServerSummary;
  token: string;
}

export interface RotateTokenResult {
  server_id: number;
  token: string;
  prefix: string;
  created_at: string;
}

export const gameServerAdmin = {
  list: (query: { search?: string; enabled?: boolean; page?: number; page_size?: number }) => {
    const params = new URLSearchParams();
    if (query.search) params.set('search', query.search);
    if (query.enabled !== undefined) params.set('enabled', String(query.enabled));
    params.set('page', String(query.page ?? 1));
    params.set('page_size', String(query.page_size ?? 20));
    return request<AdminServerList>(`/api/v1/admin/game-servers?${params.toString()}`);
  },
  detail: (id: number) => request<AdminServerDetail>(`/api/v1/admin/game-servers/${id}`),
  create: (input: CreateServerInput) =>
    request<CreateServerResult>('/api/v1/admin/game-servers', {
      method: 'POST',
      body: JSON.stringify(input),
    }),
  update: (id: number, input: Partial<CreateServerInput>) =>
    request<AdminServerSummary>(`/api/v1/admin/game-servers/${id}`, {
      method: 'PATCH',
      body: JSON.stringify(input),
    }),
  setEnabled: (id: number, enabled: boolean) =>
    request<{ server_id: number; enabled: boolean; message: string }>(
      `/api/v1/admin/game-servers/${id}/${enabled ? 'enable' : 'disable'}`,
      { method: 'POST' },
    ),
  rotateToken: (id: number) =>
    request<RotateTokenResult>(`/api/v1/admin/game-servers/${id}/token/rotate`, { method: 'POST' }),
  revokeToken: (id: number) =>
    request<{ server_id: number; token_revoked: boolean; message: string }>(
      `/api/v1/admin/game-servers/${id}/token/revoke`,
      { method: 'POST' },
    ),
  retryTask: (taskId: number) =>
    request<{ task_id: number; status: string; message: string }>(`/api/v1/admin/game-servers/tasks/${taskId}/retry`, {
      method: 'POST',
    }),
};

export const gameServerAdminKeys = {
  all: ['admin', 'game-servers'] as const,
  list: (query: unknown) => [...gameServerAdminKeys.all, 'list', query] as const,
  detail: (id: number) => [...gameServerAdminKeys.all, 'detail', id] as const,
};
