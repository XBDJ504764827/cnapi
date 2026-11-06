import { matchRoutes } from 'react-router';
import { describe, expect, it } from 'vitest';

import { router } from './index';

/**
 * Route table invariants that the guard unit test cannot see: the login page
 * must live *outside* the `/admin` branch, otherwise `RequireAdmin` would
 * redirect it to itself forever.
 */
describe('admin route table', () => {
  it('keeps /admin/login outside the guarded branch', () => {
    const loginMatches = matchRoutes(router.routes, '/admin/login');
    expect(loginMatches).not.toBeNull();
    const paths = loginMatches!.map((match) => match.route.path);
    expect(paths).toContain('/admin/login');
    // The guarded `/admin` route (and its child layout) must not be an ancestor.
    expect(paths).not.toContain('/admin');
  });

  it('guards every console page with the /admin route', () => {
    for (const path of ['/admin/dashboard', '/admin/servers', '/admin/players', '/admin/system/settings']) {
      const matches = matchRoutes(router.routes, path);
      expect(matches, path).not.toBeNull();
      expect(
        matches!.some((match) => match.route.path === '/admin'),
        `${path} is not behind the admin guard`,
      ).toBe(true);
    }
  });

  it('leaves the public portal unguarded', () => {
    for (const path of ['/', '/players', '/records']) {
      const matches = matchRoutes(router.routes, path);
      expect(matches, path).not.toBeNull();
      expect(matches!.some((match) => match.route.path === '/admin')).toBe(false);
    }
  });
});
