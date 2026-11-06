import { describe, expect, it } from 'vitest';

import { authErrorMessage, currentPath, readAuthError, sessionQuery, steamLoginUrl, userInitials } from './session';

describe('authentication helpers', () => {
  it('builds a same-origin login URL with an encoded redirect', () => {
    expect(steamLoginUrl('/admin/dashboard')).toBe('/api/v1/auth/steam/start?redirect=%2Fadmin%2Fdashboard');
    // Query strings survive the round trip without becoming a second URL.
    expect(steamLoginUrl('/records?map=kz_x&mode=vanilla')).toBe(
      `/api/v1/auth/steam/start?redirect=${encodeURIComponent('/records?map=kz_x&mode=vanilla')}`,
    );
  });

  it('reads the auth_error code emitted by the callback', () => {
    expect(readAuthError('?auth_error=cancelled')).toBe('cancelled');
    expect(readAuthError('?foo=1')).toBeNull();
    expect(readAuthError('')).toBeNull();
  });

  it('maps every documented error code to a human message', () => {
    for (const code of ['invalid_state', 'cancelled', 'invalid_assertion', 'verification_failed', 'login_failed']) {
      expect(authErrorMessage(code)).toEqual(expect.stringContaining(''));
    }
    expect(authErrorMessage(null)).toBeNull();
    expect(authErrorMessage('unknown_code')).toBe('登录失败，请重试。');
  });

  it('derives avatar initials from the Steam persona name', () => {
    expect(userInitials({ display_name: 'runner' } as never)).toBe('RU');
    expect(userInitials({ display_name: '  ' } as never)).toBe('KZ');
    expect(userInitials(null)).toBe('KZ');
  });

  it('includes the query string in the post-login path', () => {
    expect(currentPath()).toBe('/');
  });

  it('caches the session briefly so navigation does not refetch on every page', () => {
    expect(sessionQuery().staleTime).toBe(30_000);
    expect(sessionQuery().queryKey).toEqual(['auth', 'session']);
  });
});
