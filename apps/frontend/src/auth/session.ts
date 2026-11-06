import { queryOptions, useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { createApiClient } from '@cngokz/api-client';

/**
 * Authentication is the one concern shared by the public portal and the admin
 * console, so it owns its own typed client. Every call is same-origin and rides
 * on the HttpOnly session cookie; no token is ever read by JavaScript.
 */
const baseUrl = (import.meta.env.VITE_API_BASE_URL ?? '').replace(/\/+$/, '');
const api = createApiClient({ baseUrl });

/** Response of `GET /api/v1/auth/me`, derived from the generated OpenAPI schema. */
export type AuthSession = Awaited<ReturnType<typeof api.auth.getSession>>;
export type AuthUser = NonNullable<AuthSession['user']>;

export const SESSION_QUERY_KEY = ['auth', 'session'] as const;

export function sessionQuery() {
  return queryOptions({
    queryKey: SESSION_QUERY_KEY,
    queryFn: () => api.auth.getSession(),
    // The cookie is HttpOnly, so the session can only change through a login
    // redirect or a logout; a short stale window keeps navigation snappy.
    staleTime: 30_000,
    retry: false,
  });
}

/** Current Steam session; `data.authenticated` is `false` when logged out. */
export function useSession() {
  return useQuery(sessionQuery());
}

/** Revokes the session and refreshes every consumer of the session query. */
export function useLogout() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => api.auth.logout(),
    onSuccess: async () => {
      queryClient.setQueryData(SESSION_QUERY_KEY, { authenticated: false, user: null, expires_at: null });
      await queryClient.invalidateQueries({ queryKey: SESSION_QUERY_KEY });
    },
  });
}

/** Path of the current page, used as the post-login redirect target. */
export function currentPath(): string {
  // 登录失败后地址栏会带 `?auth_error=…`，它不能进入下一次登录的 redirect，
  // 否则会一路累积下去。
  const url = new URL(window.location.href);
  url.searchParams.delete('auth_error');
  return `${url.pathname}${url.search}`;
}

/**
 * Absolute URL that starts the OpenID round trip.
 *
 * A full page navigation (not `fetch`) is required: Steam redirects the browser
 * back to the API, which then sets the session cookie.
 */
export function steamLoginUrl(redirectPath: string = currentPath()): string {
  return `${baseUrl}/api/v1/auth/steam/start?redirect=${encodeURIComponent(redirectPath)}`;
}

export function startSteamLogin(redirectPath: string = currentPath()): void {
  window.location.assign(steamLoginUrl(redirectPath));
}

/**
 * Reads `?auth_error=` from the callback redirect.
 *
 * The callback never renders JSON: it always bounces back to the SPA with a
 * stable code, which is translated here.
 */
export function readAuthError(search: string = window.location.search): string | null {
  return new URLSearchParams(search).get('auth_error');
}

const AUTH_ERROR_MESSAGES: Record<string, string> = {
  invalid_state: '登录链接已失效或已被使用，请重新发起登录。',
  cancelled: '你已在 Steam 页面取消登录。',
  invalid_assertion: 'Steam 返回的登录信息未通过校验，请重试。',
  verification_failed: '无法与 Steam 校验登录状态（网络或中转服务异常），请稍后重试。',
  login_failed: '登录过程中发生错误，请重试。',
};

export function authErrorMessage(code: string | null): string | null {
  if (!code) return null;
  return AUTH_ERROR_MESSAGES[code] ?? '登录失败，请重试。';
}

/** Initials for the avatar fallback when no Steam avatar is available. */
export function userInitials(user: AuthUser | null | undefined): string {
  const name = user?.display_name?.trim();
  if (!name) return 'KZ';
  return name.slice(0, 2).toUpperCase();
}
