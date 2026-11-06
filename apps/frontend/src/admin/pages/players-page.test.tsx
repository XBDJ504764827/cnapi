import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getPlayers: vi.fn(),
}));

vi.mock('@/admin/lib/api', () => ({
  api: {
    public: {
      getPlayers: apiMocks.getPlayers,
    },
  },
}));

const { PlayersPage } = await import('./players-page');

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <PlayersPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

const payload = {
  items: [
    {
      rank: 1,
      account_id: 4885,
      steam_id64: '76561198298405388',
      display_name: '别逗你彩姐笑了',
      avatar_url: 'https://avatars.steamstatic.com/abc_medium.jpg',
      country: 'CN',
      completed_maps: 42,
      wr_count: 3,
      total_records: 57,
      last_activity_at: '2026-11-06T12:00:00Z',
      points: null,
    },
  ],
  meta: { page: 1, page_size: 20, total: 1, total_pages: 1 },
};

describe('PlayersPage', () => {
  beforeEach(() => {
    apiMocks.getPlayers.mockReset();
    apiMocks.getPlayers.mockResolvedValue(payload);
  });

  it('renders real players from the API', async () => {
    renderPage();

    expect(await screen.findByText('别逗你彩姐笑了')).toBeInTheDocument();
    expect(screen.getByText('76561198298405388')).toBeInTheDocument();
    expect(screen.getByText('42')).toBeInTheDocument();
    expect(screen.getByText('57')).toBeInTheDocument();

    // Radix 只在图片真正加载后才挂载 <img>，jsdom 不会触发 load，
    // 因此这里断言的是首字母兜底——它同样证明 PlayerCell 拿到了昵称。
    expect(screen.getByText('别逗')).toBeInTheDocument();

    // 分页信息来自 meta.total，不是本地假数据
    expect(await screen.findByText(/共 1 名玩家/)).toBeInTheDocument();
  });

  it('passes the search term to the API', async () => {
    renderPage();
    expect(await screen.findByText('别逗你彩姐笑了')).toBeInTheDocument();

    expect(apiMocks.getPlayers).toHaveBeenCalledWith(
      expect.objectContaining({ page: 1, page_size: 20, sort: 'completion' }),
    );
  });

  it('shows an explicit error state when the API fails', async () => {
    apiMocks.getPlayers.mockRejectedValue(new Error('boom'));

    renderPage();

    expect(await screen.findByText('玩家列表读取失败')).toBeInTheDocument();
  });
});
