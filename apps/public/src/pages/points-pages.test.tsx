import { beforeEach, describe, expect, it, vi } from 'vitest';

import { apiMocks, resetApiMocks } from '@/test/api-mocks';
import { playerFilterPointsFixture, playerPointsFixture, pointsLeaderboardFixture } from '@/test/fixtures';
import { renderPage } from '@/test/render';
import { screen } from '@testing-library/react';

vi.mock('@/lib/api', () => ({
  api: {
    points: {
      getPlayerPoints: apiMocks.getPlayerPoints,
      getPlayerFilterPoints: apiMocks.getPlayerFilterPoints,
      getModeLeaderboard: apiMocks.getPointsLeaderboard,
      getFilterLeaderboard: apiMocks.getFilterLeaderboard,
      getStatus: apiMocks.getPointsStatus,
      getAdminTiers: apiMocks.getAdminTiers,
      getRecalcRuns: apiMocks.getRecalcRuns,
    },
    public: {/* not used */},
  },
}));

const { PointsLeaderboardsPage } = await import('./points-leaderboards-page');
const { PlayerPointsPage } = await import('./player-points-page');

describe('points pages', () => {
  beforeEach(() => {
    resetApiMocks();
    apiMocks.getPlayerPoints.mockResolvedValue(playerPointsFixture);
    apiMocks.getPlayerFilterPoints.mockResolvedValue(playerFilterPointsFixture);
    apiMocks.getPointsLeaderboard.mockResolvedValue(pointsLeaderboardFixture);
  });

  it('renders three independent mode leaderboards', async () => {
    const view = renderPage(<PointsLeaderboardsPage />);
    expect(await view.findByText('模式积分排行榜')).toBeInTheDocument();
    expect(await view.findByText('1,500')).toBeInTheDocument();
    expect(apiMocks.getPointsLeaderboard).toHaveBeenCalledWith('KZT', { page: 1, page_size: 20 });
  });

  it('shows points leaderboard error state', async () => {
    apiMocks.getPointsLeaderboard.mockRejectedValueOnce(new Error('API down'));
    const view = renderPage(<PointsLeaderboardsPage />);
    expect(await view.findByText('无法加载积分榜')).toBeInTheDocument();
  });

  it('renders player points and Filter details', async () => {
    renderPage(<PlayerPointsPage />, {
      path: '/players/:steamid/points',
      entry: '/players/76561198000000001/points',
    });

    expect(await screen.findByText('玩家积分明细')).toBeInTheDocument();
    expect(await screen.findByText('3,000')).toBeInTheDocument();
    expect(await screen.findByText('kz_points_alpha')).toBeInTheDocument();
    expect(await screen.findByText('1,000')).toBeInTheDocument();
    expect(apiMocks.getPlayerPoints).toHaveBeenCalled();
    expect(apiMocks.getPlayerFilterPoints).toHaveBeenCalled();
  });
});
