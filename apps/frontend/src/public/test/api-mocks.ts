import { vi } from 'vitest';

/** Shared vi.fn mocks for `@/lib/api` in public portal page tests. */
export const apiMocks = {
  getStatistics: vi.fn(),
  getModes: vi.fn(),
  getLeaderboard: vi.fn(),
  getRecords: vi.fn(),
  getRecentRecords: vi.fn(),
  getRecentWorldRecords: vi.fn(),
  getRecord: vi.fn(),
  getPlayers: vi.fn(),
  getActivePlayers: vi.fn(),
  comparePlayers: vi.fn(),
  getPlayer: vi.fn(),
  getPlayerRecords: vi.fn(),
  getPlayerMaps: vi.fn(),
  getMaps: vi.fn(),
  getMap: vi.fn(),
  getMapLeaderboard: vi.fn(),
  getMapStatistics: vi.fn(),
  getServers: vi.fn(),
  getServer: vi.fn(),
  search: vi.fn(),
  getPlayerPoints: vi.fn(),
  getPlayerFilterPoints: vi.fn(),
  getPointsLeaderboard: vi.fn(),
  getModeLeaderboard: vi.fn(),
  getFilterLeaderboard: vi.fn(),
  getPointsStatus: vi.fn(),
  getAdminTiers: vi.fn(),
  getRecalcRuns: vi.fn(),
};

export function resetApiMocks() {
  for (const mock of Object.values(apiMocks)) mock.mockReset();
}
