import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter, Navigate } from 'react-router';

import { PageLoading } from '@/components/common/page-loading';
import { AppLayout } from '@/layouts/app-layout';
import { NotFoundPage } from '@/pages/not-found-page';
import { RouteErrorPage } from '@/pages/route-error-page';

// Route level code splitting: one chunk per page.
const DashboardPage = lazy(async () => ({ default: (await import('@/pages/dashboard-page')).DashboardPage }));
const SystemStatusPage = lazy(async () => ({ default: (await import('@/pages/system-status-page')).SystemStatusPage }));
const PointsPage = lazy(async () => ({ default: (await import('@/pages/points-page')).PointsPage }));
const PointsTiersPage = lazy(async () => ({ default: (await import('@/pages/points-tiers-page')).PointsTiersPage }));
const PointsRecalcPage = lazy(async () => ({ default: (await import('@/pages/points-recalc-page')).PointsRecalcPage }));
const PlayersPage = lazy(async () => ({ default: (await import('@/pages/players-page')).PlayersPage }));
const PlayerDetailPage = lazy(async () => ({ default: (await import('@/pages/player-detail-page')).PlayerDetailPage }));
const PlayerPointsPage = lazy(async () => ({ default: (await import('@/pages/player-points-page')).PlayerPointsPage }));
const MapsPage = lazy(async () => ({ default: (await import('@/pages/maps-page')).MapsPage }));
const RecordsPage = lazy(async () => ({ default: (await import('@/pages/records-page')).RecordsPage }));
const LeaderboardPage = lazy(async () => ({ default: (await import('@/pages/leaderboard-page')).LeaderboardPage }));
const JumpstatsPage = lazy(async () => ({ default: (await import('@/pages/jumpstats-page')).JumpstatsPage }));
const ReplaysPage = lazy(async () => ({ default: (await import('@/pages/replays-page')).ReplaysPage }));
const ServersPage = lazy(async () => ({ default: (await import('@/pages/servers-page')).ServersPage }));
const SourcesPage = lazy(async () => ({ default: (await import('@/pages/sources-page')).SourcesPage }));
const ServerDetailPage = lazy(async () => ({ default: (await import('@/pages/server-detail-page')).ServerDetailPage }));
const BansPage = lazy(async () => ({ default: (await import('@/pages/bans-page')).BansPage }));
const ApiKeysPage = lazy(async () => ({ default: (await import('@/pages/api-keys-page')).ApiKeysPage }));
const AuditLogsPage = lazy(async () => ({ default: (await import('@/pages/audit-logs-page')).AuditLogsPage }));
const ApiDocsPage = lazy(async () => ({ default: (await import('@/pages/api-docs-page')).ApiDocsPage }));
const RequestLogsPage = lazy(async () => ({ default: (await import('@/pages/request-logs-page')).RequestLogsPage }));
const SettingsPage = lazy(async () => ({ default: (await import('@/pages/settings-page')).SettingsPage }));

/**
 * Paths must stay in sync with `src/components/layout/nav-config.ts`.
 * Legacy phase 1/2 paths redirect to the new information architecture.
 */
export const router = createBrowserRouter(
  [
    {
      path: '/',
      element: <AppLayout />,
      errorElement: <RouteErrorPage />,
      children: [
        { index: true, element: <Navigate to="/dashboard" replace /> },

        { path: 'dashboard', element: withSuspense(<DashboardPage />) },
        { path: 'system/status', element: withSuspense(<SystemStatusPage />) },
        { path: 'points', element: withSuspense(<PointsPage />) },
        { path: 'points/tiers', element: withSuspense(<PointsTiersPage />) },
        { path: 'points/recalc', element: withSuspense(<PointsRecalcPage />) },
        { path: 'system/settings', element: withSuspense(<SettingsPage />) },

        { path: 'players', element: withSuspense(<PlayersPage />) },
        { path: 'players/:playerId', element: withSuspense(<PlayerDetailPage />) },
        { path: 'players/:playerId/points', element: withSuspense(<PlayerPointsPage />) },

        { path: 'maps', element: withSuspense(<MapsPage />) },
        { path: 'records', element: withSuspense(<RecordsPage />) },
        { path: 'leaderboard', element: withSuspense(<LeaderboardPage />) },
        { path: 'jumpstats', element: withSuspense(<JumpstatsPage />) },
        { path: 'replays', element: withSuspense(<ReplaysPage />) },

        { path: 'servers', element: withSuspense(<ServersPage />) },
        { path: 'sources', element: withSuspense(<SourcesPage />) },
        { path: 'servers/:serverId', element: withSuspense(<ServerDetailPage />) },

        { path: 'bans', element: withSuspense(<BansPage />) },
        { path: 'audit-logs', element: withSuspense(<AuditLogsPage />) },

        { path: 'developer/api-keys', element: withSuspense(<ApiKeysPage />) },
        { path: 'developer/api-docs', element: withSuspense(<ApiDocsPage />) },
        { path: 'developer/request-logs', element: withSuspense(<RequestLogsPage />) },

        // Phase 1/2 route compatibility.
        { path: 'kz/players', element: <Navigate to="/players" replace /> },
        { path: 'kz/maps', element: <Navigate to="/maps" replace /> },
        { path: 'kz/records', element: <Navigate to="/records" replace /> },
        { path: 'kz/leaderboard', element: <Navigate to="/leaderboard" replace /> },
        { path: 'data/jumpstats', element: <Navigate to="/jumpstats" replace /> },
        { path: 'data/replays', element: <Navigate to="/replays" replace /> },
        { path: 'data/bans', element: <Navigate to="/bans" replace /> },
        { path: 'servers/status', element: <Navigate to="/servers" replace /> },

        { path: '*', element: <NotFoundPage /> },
      ],
    },
  ],
  { basename: '/admin' },
);

function withSuspense(element: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{element}</Suspense>;
}
