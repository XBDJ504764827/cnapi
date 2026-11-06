import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter, Navigate } from 'react-router';

import { AppProviders } from '@/providers';
import { RequireAdmin } from '@/admin/components/auth/require-admin';
import { PageLoading as AdminPageLoading } from '@/admin/components/common/page-loading';
import { AppLayout } from '@/admin/layouts/app-layout';
import { NotFoundPage as AdminNotFoundPage } from '@/admin/pages/not-found-page';
import { RouteErrorPage as AdminRouteErrorPage } from '@/admin/pages/route-error-page';
import { PageLoading as PublicPageLoading } from '@/public/components/common/page-loading';
import { PublicLayout } from '@/public/layouts/public-layout';
import { NotFoundPage as PublicNotFoundPage } from '@/public/pages/not-found-page';
import { RouteErrorPage as PublicRouteErrorPage } from '@/public/pages/route-error-page';

const Admin = {
  Login: lazy(async () => ({ default: (await import('@/admin/pages/login-page')).LoginPage })),
  Dashboard: lazy(async () => ({ default: (await import('@/admin/pages/dashboard-page')).DashboardPage })),
  SystemStatus: lazy(async () => ({ default: (await import('@/admin/pages/system-status-page')).SystemStatusPage })),
  Points: lazy(async () => ({ default: (await import('@/admin/pages/points-page')).PointsPage })),
  PointsTiers: lazy(async () => ({ default: (await import('@/admin/pages/points-tiers-page')).PointsTiersPage })),
  PointsRecalc: lazy(async () => ({ default: (await import('@/admin/pages/points-recalc-page')).PointsRecalcPage })),
  Players: lazy(async () => ({ default: (await import('@/admin/pages/players-page')).PlayersPage })),
  PlayerDetail: lazy(async () => ({ default: (await import('@/admin/pages/player-detail-page')).PlayerDetailPage })),
  PlayerPoints: lazy(async () => ({ default: (await import('@/admin/pages/player-points-page')).PlayerPointsPage })),
  Maps: lazy(async () => ({ default: (await import('@/admin/pages/maps-page')).MapsPage })),
  Records: lazy(async () => ({ default: (await import('@/admin/pages/records-page')).RecordsPage })),
  Leaderboard: lazy(async () => ({ default: (await import('@/admin/pages/leaderboard-page')).LeaderboardPage })),
  Jumpstats: lazy(async () => ({ default: (await import('@/admin/pages/jumpstats-page')).JumpstatsPage })),
  Replays: lazy(async () => ({ default: (await import('@/admin/pages/replays-page')).ReplaysPage })),
  Servers: lazy(async () => ({ default: (await import('@/admin/pages/servers-page')).ServersPage })),
  Sources: lazy(async () => ({ default: (await import('@/admin/pages/sources-page')).SourcesPage })),
  ServerDetail: lazy(async () => ({ default: (await import('@/admin/pages/server-detail-page')).ServerDetailPage })),
  Bans: lazy(async () => ({ default: (await import('@/admin/pages/bans-page')).BansPage })),
  ApiKeys: lazy(async () => ({ default: (await import('@/admin/pages/api-keys-page')).ApiKeysPage })),
  AuditLogs: lazy(async () => ({ default: (await import('@/admin/pages/audit-logs-page')).AuditLogsPage })),
  ApiDocs: lazy(async () => ({ default: (await import('@/admin/pages/api-docs-page')).ApiDocsPage })),
  RequestLogs: lazy(async () => ({ default: (await import('@/admin/pages/request-logs-page')).RequestLogsPage })),
  Settings: lazy(async () => ({ default: (await import('@/admin/pages/settings-page')).SettingsPage })),
};

const Public = {
  Home: lazy(async () => ({ default: (await import('@/public/pages/home-page')).HomePage })),
  Leaderboards: lazy(async () => ({ default: (await import('@/public/pages/leaderboards-page')).LeaderboardsPage })),
  Players: lazy(async () => ({ default: (await import('@/public/pages/players-page')).PlayersPage })),
  PlayerDetail: lazy(async () => ({ default: (await import('@/public/pages/player-detail-page')).PlayerDetailPage })),
  Compare: lazy(async () => ({ default: (await import('@/public/pages/compare-page')).ComparePage })),
  Maps: lazy(async () => ({ default: (await import('@/public/pages/maps-page')).MapsPage })),
  MapDetail: lazy(async () => ({ default: (await import('@/public/pages/map-detail-page')).MapDetailPage })),
  Records: lazy(async () => ({ default: (await import('@/public/pages/records-page')).RecordsPage })),
  Servers: lazy(async () => ({ default: (await import('@/public/pages/servers-page')).ServersPage })),
  PointsLeaderboards: lazy(async () => ({
    default: (await import('@/public/pages/points-leaderboards-page')).PointsLeaderboardsPage,
  })),
  PlayerPoints: lazy(async () => ({ default: (await import('@/public/pages/player-points-page')).PlayerPointsPage })),
};

const adminLazy = (element: ReactNode) => <Suspense fallback={<AdminPageLoading />}>{element}</Suspense>;
const publicLazy = (element: ReactNode) => <Suspense fallback={<PublicPageLoading />}>{element}</Suspense>;

export const router = createBrowserRouter([
  {
    element: <AppProviders />,
    children: [
      {
        path: '/',
        element: <PublicLayout />,
        errorElement: <PublicRouteErrorPage />,
        children: [
          { index: true, element: publicLazy(<Public.Home />) },
          { path: 'leaderboards', element: publicLazy(<Public.Leaderboards />) },
          { path: 'players', element: publicLazy(<Public.Players />) },
          { path: 'players/:steamid', element: publicLazy(<Public.PlayerDetail />) },
          { path: 'players/:steamid/points', element: publicLazy(<Public.PlayerPoints />) },
          { path: 'compare', element: publicLazy(<Public.Compare />) },
          { path: 'maps', element: publicLazy(<Public.Maps />) },
          { path: 'maps/:slug', element: publicLazy(<Public.MapDetail />) },
          { path: 'records', element: publicLazy(<Public.Records />) },
          { path: 'servers', element: publicLazy(<Public.Servers />) },
          { path: 'points', element: publicLazy(<Public.PointsLeaderboards />) },
          { path: '*', element: <PublicNotFoundPage /> },
        ],
      },
      {
        // Sign-in lives outside the guarded tree: it has to render for
        // anonymous visitors, and it never shows the console shell.
        path: '/admin/login',
        element: adminLazy(<Admin.Login />),
        errorElement: <AdminRouteErrorPage />,
      },
      {
        // Everything under `/admin` requires an authenticated administrator.
        path: '/admin',
        element: <RequireAdmin />,
        errorElement: <AdminRouteErrorPage />,
        children: [
          {
            element: <AppLayout />,
            children: [
              { index: true, element: <Navigate to="/admin/dashboard" replace /> },
              { path: 'dashboard', element: adminLazy(<Admin.Dashboard />) },
              { path: 'system/status', element: adminLazy(<Admin.SystemStatus />) },
              { path: 'points', element: adminLazy(<Admin.Points />) },
              { path: 'points/tiers', element: adminLazy(<Admin.PointsTiers />) },
              { path: 'points/recalc', element: adminLazy(<Admin.PointsRecalc />) },
              { path: 'system/settings', element: adminLazy(<Admin.Settings />) },
              { path: 'players', element: adminLazy(<Admin.Players />) },
              { path: 'players/:playerId', element: adminLazy(<Admin.PlayerDetail />) },
              { path: 'players/:playerId/points', element: adminLazy(<Admin.PlayerPoints />) },
              { path: 'maps', element: adminLazy(<Admin.Maps />) },
              { path: 'records', element: adminLazy(<Admin.Records />) },
              { path: 'leaderboard', element: adminLazy(<Admin.Leaderboard />) },
              { path: 'jumpstats', element: adminLazy(<Admin.Jumpstats />) },
              { path: 'replays', element: adminLazy(<Admin.Replays />) },
              { path: 'servers', element: adminLazy(<Admin.Servers />) },
              { path: 'sources', element: adminLazy(<Admin.Sources />) },
              { path: 'servers/:serverId', element: adminLazy(<Admin.ServerDetail />) },
              { path: 'bans', element: adminLazy(<Admin.Bans />) },
              { path: 'audit-logs', element: adminLazy(<Admin.AuditLogs />) },
              { path: 'developer/api-keys', element: adminLazy(<Admin.ApiKeys />) },
              { path: 'developer/api-docs', element: adminLazy(<Admin.ApiDocs />) },
              { path: 'developer/request-logs', element: adminLazy(<Admin.RequestLogs />) },
              { path: '*', element: <AdminNotFoundPage /> },
            ],
          },
        ],
      },
    ],
  },
]);
