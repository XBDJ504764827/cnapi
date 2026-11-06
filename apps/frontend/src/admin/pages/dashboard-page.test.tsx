import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getStatus: vi.fn(),
  getInfo: vi.fn(),
  getRecentEvents: vi.fn(),
  getStatistics: vi.fn(),
}));

vi.mock('@/admin/lib/api', () => ({
  api: {
    system: {
      getStatus: apiMocks.getStatus,
      getInfo: apiMocks.getInfo,
      getRecentEvents: apiMocks.getRecentEvents,
    },
    public: {
      getStatistics: apiMocks.getStatistics,
    },
  },
}));

const { DashboardPage } = await import('./dashboard-page');

function renderDashboard() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <DashboardPage />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

const statusPayload = {
  status: 'healthy',
  version: '0.3.1',
  environment: 'production',
  started_at: '2026-11-03T10:00:00Z',
  uptime_seconds: 90,
  checked_at: '2026-11-03T10:05:00Z',
  checks: [
    { name: 'postgresql', status: 'up', latency_ms: 3, detail: null },
    { name: 'redis', status: 'up', latency_ms: 1, detail: null },
  ],
};

const infoPayload = {
  name: 'CNGOKZAPI',
  version: '0.3.1',
  environment: 'production',
  api_version: 'v1',
  started_at: '2026-11-03T10:00:00Z',
  uptime_seconds: 90,
};

describe('DashboardPage', () => {
  beforeEach(() => {
    apiMocks.getStatus.mockResolvedValue(statusPayload);
    apiMocks.getInfo.mockResolvedValue(infoPayload);
    apiMocks.getRecentEvents.mockResolvedValue({ items: [], count: 0, limit: 10 });
    apiMocks.getStatistics.mockResolvedValue({
      players: 388,
      maps: 990,
      courses: 1515,
      records: 4884,
      world_records: 1200,
      servers_online: 2,
      generated_at: '2026-11-03T10:05:00Z',
    });
  });

  it('renders dependency status and system information from the API', async () => {
    renderDashboard();

    expect(await screen.findByText('PostgreSQL')).toBeInTheDocument();
    expect(await screen.findByText('Redis')).toBeInTheDocument();
    expect(await screen.findByText('3 ms')).toBeInTheDocument();
    expect(await screen.findByText(/1 分 30 秒/)).toBeInTheDocument();
    expect((await screen.findAllByText(/0\.3\.1/)).length).toBeGreaterThanOrEqual(1);
    expect(await screen.findByText('暂无系统事件')).toBeInTheDocument();
  });

  it('renders the four KPI cards from /public/statistics', async () => {
    renderDashboard();

    expect(await screen.findByText('玩家总数')).toBeInTheDocument();
    expect(await screen.findByText('388')).toBeInTheDocument();
    expect(await screen.findByText('990')).toBeInTheDocument();
    expect(await screen.findByText('4,884')).toBeInTheDocument();
    expect(await screen.findByText('1,515 条课程')).toBeInTheDocument();
    // 卡片的 hint 用的是真实统计，不再是「接口未接入」
    expect(await screen.findByText('心跳 120 秒内的服务器')).toBeInTheDocument();
  });

  it('falls back to dashes when the statistics endpoint fails', async () => {
    apiMocks.getStatistics.mockRejectedValue(new Error('boom'));

    renderDashboard();

    expect(await screen.findByText('玩家总数')).toBeInTheDocument();
    // 等待查询进入 error 状态：卡片退化为「—」并说明原因，而不是编造数字
    expect((await screen.findAllByText('统计接口不可用')).length).toBeGreaterThanOrEqual(4);
    expect(screen.getAllByText('—').length).toBeGreaterThanOrEqual(4);
  });

  it('shows an error state when the API is unreachable', async () => {
    apiMocks.getStatus.mockRejectedValue(new Error('Unable to reach the API'));

    renderDashboard();

    expect(await screen.findByText('无法获取系统状态')).toBeInTheDocument();
  });
});
