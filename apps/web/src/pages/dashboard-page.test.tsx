import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const apiMocks = vi.hoisted(() => ({
  getStatus: vi.fn(),
  getInfo: vi.fn(),
  getRecentEvents: vi.fn(),
}));

vi.mock('@/lib/api', () => ({
  api: {
    system: {
      getStatus: apiMocks.getStatus,
      getInfo: apiMocks.getInfo,
      getRecentEvents: apiMocks.getRecentEvents,
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
      <DashboardPage />
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
  });

  it('renders dependency status and system information from the API', async () => {
    renderDashboard();

    expect(await screen.findByText('PostgreSQL')).toBeInTheDocument();
    expect(await screen.findByText('Redis')).toBeInTheDocument();
    expect(await screen.findByText('3 ms')).toBeInTheDocument();
    expect(await screen.findByText('1 分 30 秒')).toBeInTheDocument();
    expect((await screen.findAllByText('0.3.1')).length).toBeGreaterThanOrEqual(1);
    expect(await screen.findByText('暂无系统事件')).toBeInTheDocument();
  });

  it('keeps aggregate cards empty instead of inventing numbers', async () => {
    renderDashboard();

    expect(await screen.findByText('玩家数量')).toBeInTheDocument();
    expect(screen.getAllByText('—').length).toBeGreaterThanOrEqual(4);
    expect(screen.getAllByText('统计接口将在后续阶段接入')).toHaveLength(4);
  });

  it('shows an error state when the API is unreachable', async () => {
    apiMocks.getStatus.mockRejectedValue(new Error('Unable to reach the API'));

    renderDashboard();

    expect(await screen.findByText('无法获取系统状态')).toBeInTheDocument();
  });
});
