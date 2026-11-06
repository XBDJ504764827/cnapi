import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render, screen } from '@testing-library/react';
import { createMemoryRouter, RouterProvider } from 'react-router';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const sessionMocks = vi.hoisted(() => ({
  useSession: vi.fn(),
  useLogout: vi.fn(() => ({ mutate: vi.fn(), isPending: false })),
  startSteamLogin: vi.fn(),
  userInitials: vi.fn(() => 'KZ'),
}));

vi.mock('@/auth/session', () => ({
  useSession: sessionMocks.useSession,
  useLogout: sessionMocks.useLogout,
  startSteamLogin: sessionMocks.startSteamLogin,
  userInitials: sessionMocks.userInitials,
}));

const { RequireAdmin } = await import('./require-admin');

function renderAt(path: string) {
  const router = createMemoryRouter(
    [
      {
        path: '/admin',
        element: <RequireAdmin />,
        children: [{ index: true, element: <div>控制台内容</div> }],
      },
      { path: '/admin/login', element: <div>登录页面</div> },
    ],
    { initialEntries: [path] },
  );

  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

const adminSession = {
  authenticated: true,
  expires_at: '2026-12-06T00:00:00Z',
  user: {
    account_id: 1,
    steam_id64: '76561198000000901',
    display_name: 'Admin',
    avatar_url: null,
    is_admin: true,
    roles: ['admin'],
    steam_profile_url: 'https://steamcommunity.com/profiles/76561198000000901',
  },
};

describe('RequireAdmin', () => {
  beforeEach(() => {
    sessionMocks.useSession.mockReset();
  });

  it('sends anonymous visitors to the login page', async () => {
    sessionMocks.useSession.mockReturnValue({
      data: { authenticated: false, user: null, expires_at: null },
      isPending: false,
      isError: false,
    });

    renderAt('/admin');
    expect(await screen.findByText('登录页面')).toBeInTheDocument();
    expect(screen.queryByText('控制台内容')).not.toBeInTheDocument();
  });

  it('shows a 403 instead of the console for logged-in players', async () => {
    sessionMocks.useSession.mockReturnValue({
      data: {
        ...adminSession,
        user: { ...adminSession.user, is_admin: false, roles: [] },
      },
      isPending: false,
      isError: false,
    });

    renderAt('/admin');
    expect(await screen.findByText('没有管理后台访问权限')).toBeInTheDocument();
    expect(screen.queryByText('控制台内容')).not.toBeInTheDocument();
  });

  it('renders the console for administrators', async () => {
    sessionMocks.useSession.mockReturnValue({
      data: adminSession,
      isPending: false,
      isError: false,
    });

    renderAt('/admin');
    expect(await screen.findByText('控制台内容')).toBeInTheDocument();
  });

  it('waits instead of flashing the login page while the session loads', () => {
    sessionMocks.useSession.mockReturnValue({ data: undefined, isPending: true, isError: false });

    renderAt('/admin');
    expect(screen.getByRole('status', { name: '正在验证登录状态' })).toBeInTheDocument();
    expect(screen.queryByText('登录页面')).not.toBeInTheDocument();
  });
});
