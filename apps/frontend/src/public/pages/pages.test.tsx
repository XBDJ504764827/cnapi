import { screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { apiMocks, resetApiMocks } from '@/public/test/api-mocks';
import {
  compareFixture,
  leaderboardFixture,
  mapDetailFixture,
  mapSummaryFixture,
  modesFixture,
  playerDetailFixture,
  playerMapsFixture,
  playerSummaryFixture,
  recordFixture,
  searchFixture,
  statisticsFixture,
} from '@/public/test/fixtures';
import { renderPage } from '@/public/test/render';

vi.mock('@/public/lib/api', () => ({
  api: {
    public: {
      getStatistics: apiMocks.getStatistics,
      getModes: apiMocks.getModes,
      getLeaderboard: apiMocks.getLeaderboard,
      getRecords: apiMocks.getRecords,
      getRecentRecords: apiMocks.getRecentRecords,
      getRecentWorldRecords: apiMocks.getRecentWorldRecords,
      getRecord: apiMocks.getRecord,
      getPlayers: apiMocks.getPlayers,
      getActivePlayers: apiMocks.getActivePlayers,
      comparePlayers: apiMocks.comparePlayers,
      getPlayer: apiMocks.getPlayer,
      getPlayerRecords: apiMocks.getPlayerRecords,
      getPlayerMaps: apiMocks.getPlayerMaps,
      getMaps: apiMocks.getMaps,
      getMap: apiMocks.getMap,
      getMapLeaderboard: apiMocks.getMapLeaderboard,
      getMapStatistics: apiMocks.getMapStatistics,
      getServers: apiMocks.getServers,
      getServer: apiMocks.getServer,
      search: apiMocks.search,
    },
  },
}));

const { HomePage } = await import('./home-page');
const { LeaderboardsPage } = await import('./leaderboards-page');
const { PlayersPage } = await import('./players-page');
const { PlayerDetailPage } = await import('./player-detail-page');
const { MapsPage } = await import('./maps-page');
const { MapDetailPage } = await import('./map-detail-page');
const { RecordsPage } = await import('./records-page');
const { ServersPage } = await import('./servers-page');
const { ComparePage } = await import('./compare-page');

const emptyList = () => ({ items: [], meta: { page: 1, page_size: 20, total: 0, total_pages: 0 } });

function mockAllEmpty() {
  apiMocks.getStatistics.mockResolvedValue({
    ...statisticsFixture,
    players: 0,
    maps: 0,
    records: 0,
    world_records: 0,
    servers_online: 0,
  });
  apiMocks.getModes.mockResolvedValue(modesFixture);
  apiMocks.getLeaderboard.mockResolvedValue(leaderboardFixture());
  apiMocks.getRecentRecords.mockResolvedValue({ items: [], count: 0, limit: 20 });
  apiMocks.getRecentWorldRecords.mockResolvedValue([]);
  apiMocks.getActivePlayers.mockResolvedValue({ items: [], days: 7 });
  apiMocks.getServers.mockResolvedValue(emptyList());
  apiMocks.getPlayers.mockResolvedValue({
    mode: null,
    ranking_metric: 'completed_maps',
    points_available: false,
    ...emptyList(),
  });
  apiMocks.getPlayer.mockResolvedValue(playerDetailFixture);
  apiMocks.getPlayerRecords.mockResolvedValue({ account: playerDetailFixture.profile, ...emptyList() });
  apiMocks.getPlayerMaps.mockResolvedValue(playerMapsFixture);
  apiMocks.getMaps.mockResolvedValue({ mode: modesFixture.items[0], time_type: 'all', ...emptyList() });
  apiMocks.getMap.mockResolvedValue(mapDetailFixture);
  apiMocks.getMapLeaderboard.mockResolvedValue({
    map: { id: 1, name: 'kz_fixture_alpha', slug: 'kz_fixture_alpha' },
    course: { id: 1, stage: 0, name: 'Main' },
    mode: modesFixture.items[0],
    style: { id: 1, code: 'normal', display_name: 'Normal' },
    time_type: 'pro',
    ...emptyList(),
  });
  apiMocks.getMapStatistics.mockResolvedValue(mapDetailFixture.statistics);
  apiMocks.getRecords.mockResolvedValue({ ...emptyList() });
  apiMocks.search.mockResolvedValue(searchFixture);
}

describe('public portal pages', () => {
  beforeEach(() => {
    resetApiMocks();
    mockAllEmpty();
  });

  it('renders the home page with real statistics and empty data states', async () => {
    renderPage(<HomePage />);

    expect(await screen.findByText('数据概览')).toBeInTheDocument();
    expect(await screen.findByText('暂无排行榜数据')).toBeInTheDocument();
    expect(await screen.findByText('暂无成绩')).toBeInTheDocument();
    expect(apiMocks.getStatistics).toHaveBeenCalled();
    expect(apiMocks.getRecentRecords).toHaveBeenCalled();
  });

  it('renders leaderboard success and error states', async () => {
    apiMocks.getLeaderboard.mockResolvedValueOnce(leaderboardFixture([playerSummaryFixture]));
    const success = renderPage(<LeaderboardsPage />);
    expect(await success.findByText('Alpha')).toBeInTheDocument();
    expect(await success.findByText('Vanilla 排名')).toBeInTheDocument();

    apiMocks.getLeaderboard.mockRejectedValueOnce(new Error('API unavailable'));
    const failed = renderPage(<LeaderboardsPage />);
    expect(await failed.findByText('无法加载排行榜')).toBeInTheDocument();
  });

  it('renders player detail with scoped and unscoped statistics', async () => {
    renderPage(<PlayerDetailPage />, { path: '/players/:steamid', entry: '/players/76561198000000001' });

    expect(await screen.findByText('Alpha')).toBeInTheDocument();
    expect(await screen.findByText('当前范围')).toBeInTheDocument();
    expect(await screen.findByText('难度完成进度')).toBeInTheDocument();
    expect(await screen.findByText('kz_fixture_alpha')).toBeInTheDocument();
    expect(apiMocks.getPlayer).toHaveBeenCalled();
    expect(apiMocks.getPlayerMaps).toHaveBeenCalled();
  });

  it('renders player list empty and success states', async () => {
    const empty = renderPage(<PlayersPage />);
    expect(await empty.findByText('暂无玩家数据')).toBeInTheDocument();

    apiMocks.getPlayers.mockResolvedValueOnce({
      mode: null,
      ranking_metric: 'completed_maps',
      points_available: false,
      items: [playerSummaryFixture],
      meta: { page: 1, page_size: 20, total: 1, total_pages: 1 },
    });
    const success = renderPage(<PlayersPage />);
    expect(await success.findByText('Alpha')).toBeInTheDocument();
  });

  it('renders map list and map detail with the same map data', async () => {
    apiMocks.getMaps.mockResolvedValueOnce({
      mode: modesFixture.items[0],
      time_type: 'all',
      items: [mapSummaryFixture],
      meta: { page: 1, page_size: 20, total: 1, total_pages: 1 },
    });
    const list = renderPage(<MapsPage />);
    expect(await list.findByText('kz_fixture_alpha')).toBeInTheDocument();

    renderPage(<MapDetailPage />, { path: '/maps/:slug', entry: '/maps/kz_fixture_alpha' });
    expect(await screen.findByRole('heading', { level: 1, name: 'kz_fixture_alpha' })).toBeInTheDocument();
  });

  it('renders records empty state and a real record row', async () => {
    const empty = renderPage(<RecordsPage />);
    expect(await empty.findByText('暂无成绩')).toBeInTheDocument();

    apiMocks.getRecords.mockResolvedValueOnce({
      items: [recordFixture],
      meta: { page: 1, page_size: 20, total: 1, total_pages: 1 },
    });
    const success = renderPage(<RecordsPage />);
    expect(await success.findByText('Alpha')).toBeInTheDocument();
    expect(await success.findByText('12.500')).toBeInTheDocument();
  });

  it('renders server empty state and heartbeat classification fields', async () => {
    const empty = renderPage(<ServersPage />);
    expect(await empty.findByText('暂无服务器数据')).toBeInTheDocument();

    apiMocks.getServers.mockResolvedValueOnce({
      items: [
        {
          id: 1,
          name: 'Alpha Server',
          community: 'Alpha',
          ip: '10.0.0.1',
          port: 27015,
          online: true,
          status: 'online',
          heartbeat_age_seconds: 5,
          current_map_id: 1,
          current_map_name: 'kz_fixture_alpha',
          current_map_slug: 'kz_fixture_alpha',
          plugin_version: '1.2.3',
          last_heartbeat_at: '2026-11-04T12:00:00Z',
          players: null,
          max_players: null,
          mode: null,
        },
      ],
      meta: { page: 1, page_size: 20, total: 1, total_pages: 1 },
    });
    const success = renderPage(<ServersPage />);
    expect(await success.findByText('Alpha Server')).toBeInTheDocument();
    expect(await success.findByText('在线', { selector: 'span' })).toBeInTheDocument();
  });

  it('renders compare with two players and map comparison', async () => {
    apiMocks.comparePlayers.mockResolvedValueOnce(compareFixture);
    renderPage(<ComparePage />, {
      entry: '/compare?left=76561198000000001&right=76561198000000002',
    });

    expect(await screen.findByText('玩家比较')).toBeInTheDocument();
    expect(await screen.findByText('综合数据')).toBeInTheDocument();
    expect(await screen.findByText('kz_fixture_alpha')).toBeInTheDocument();
    expect(apiMocks.comparePlayers).toHaveBeenCalled();
  });

  it('renders a 404 page', async () => {
    const { NotFoundPage } = await import('./not-found-page');
    const view = renderPage(<NotFoundPage />);
    expect(await view.findByText('页面不存在')).toBeInTheDocument();
  });
});
