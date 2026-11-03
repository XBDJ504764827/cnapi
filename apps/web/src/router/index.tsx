import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter, Navigate } from 'react-router';

import { PageLoading } from '@/components/common/page-loading';
import { AppLayout } from '@/layouts/app-layout';
import { NotFoundPage } from '@/pages/not-found-page';
import { RouteErrorPage } from '@/pages/route-error-page';

// Route level code splitting: one chunk per page.
const DashboardPage = lazy(async () => ({ default: (await import('@/pages/dashboard-page')).DashboardPage }));
const PlayersPage = lazy(async () => ({ default: (await import('@/pages/players-page')).PlayersPage }));
const MapsPage = lazy(async () => ({ default: (await import('@/pages/maps-page')).MapsPage }));
const RecordsPage = lazy(async () => ({ default: (await import('@/pages/records-page')).RecordsPage }));
const LeaderboardPage = lazy(async () => ({ default: (await import('@/pages/leaderboard-page')).LeaderboardPage }));
const ServersPage = lazy(async () => ({ default: (await import('@/pages/servers-page')).ServersPage }));
const ServerStatusPage = lazy(async () => ({ default: (await import('@/pages/server-status-page')).ServerStatusPage }));
const JumpstatsPage = lazy(async () => ({ default: (await import('@/pages/jumpstats-page')).JumpstatsPage }));
const ReplaysPage = lazy(async () => ({ default: (await import('@/pages/replays-page')).ReplaysPage }));
const BansPage = lazy(async () => ({ default: (await import('@/pages/bans-page')).BansPage }));
const ApiDocsPage = lazy(async () => ({ default: (await import('@/pages/api-docs-page')).ApiDocsPage }));
const ApiKeysPage = lazy(async () => ({ default: (await import('@/pages/api-keys-page')).ApiKeysPage }));
const SettingsPage = lazy(async () => ({ default: (await import('@/pages/settings-page')).SettingsPage }));
const SystemStatusPage = lazy(async () => ({ default: (await import('@/pages/system-status-page')).SystemStatusPage }));

/**
 * Paths must stay in sync with `src/components/layout/nav-config.ts`.
 */
export const router = createBrowserRouter([
  {
    path: '/',
    element: <AppLayout />,
    errorElement: <RouteErrorPage />,
    children: [
      { index: true, element: <Navigate to="/dashboard" replace /> },
      { path: 'dashboard', element: withSuspense(<DashboardPage />) },
      { path: 'kz/players', element: withSuspense(<PlayersPage />) },
      { path: 'kz/maps', element: withSuspense(<MapsPage />) },
      { path: 'kz/records', element: withSuspense(<RecordsPage />) },
      { path: 'kz/leaderboard', element: withSuspense(<LeaderboardPage />) },
      { path: 'servers', element: withSuspense(<ServersPage />) },
      { path: 'servers/status', element: withSuspense(<ServerStatusPage />) },
      { path: 'data/jumpstats', element: withSuspense(<JumpstatsPage />) },
      { path: 'data/replays', element: withSuspense(<ReplaysPage />) },
      { path: 'data/bans', element: withSuspense(<BansPage />) },
      { path: 'developer/api-docs', element: withSuspense(<ApiDocsPage />) },
      { path: 'developer/api-keys', element: withSuspense(<ApiKeysPage />) },
      { path: 'system/settings', element: withSuspense(<SettingsPage />) },
      { path: 'system/status', element: withSuspense(<SystemStatusPage />) },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);

function withSuspense(element: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{element}</Suspense>;
}
