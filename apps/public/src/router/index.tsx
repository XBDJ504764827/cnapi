import { lazy, Suspense, type ReactNode } from 'react';
import { createBrowserRouter } from 'react-router';

import { PageLoading } from '@/components/common/page-loading';
import { PublicLayout } from '@/layouts/public-layout';
import { NotFoundPage } from '@/pages/not-found-page';
import { RouteErrorPage } from '@/pages/route-error-page';

const HomePage = lazy(async () => ({ default: (await import('@/pages/home-page')).HomePage }));
const LeaderboardsPage = lazy(async () => ({ default: (await import('@/pages/leaderboards-page')).LeaderboardsPage }));
const PlayersPage = lazy(async () => ({ default: (await import('@/pages/players-page')).PlayersPage }));
const PlayerDetailPage = lazy(async () => ({ default: (await import('@/pages/player-detail-page')).PlayerDetailPage }));
const ComparePage = lazy(async () => ({ default: (await import('@/pages/compare-page')).ComparePage }));
const MapsPage = lazy(async () => ({ default: (await import('@/pages/maps-page')).MapsPage }));
const MapDetailPage = lazy(async () => ({ default: (await import('@/pages/map-detail-page')).MapDetailPage }));
const RecordsPage = lazy(async () => ({ default: (await import('@/pages/records-page')).RecordsPage }));
const ServersPage = lazy(async () => ({ default: (await import('@/pages/servers-page')).ServersPage }));
const PointsLeaderboardsPage = lazy(async () => ({
  default: (await import('@/pages/points-leaderboards-page')).PointsLeaderboardsPage,
}));
const PlayerPointsPage = lazy(async () => ({ default: (await import('@/pages/player-points-page')).PlayerPointsPage }));

export const router = createBrowserRouter([
  {
    path: '/',
    element: <PublicLayout />,
    errorElement: <RouteErrorPage />,
    children: [
      { index: true, element: withSuspense(<HomePage />) },
      { path: 'leaderboards', element: withSuspense(<LeaderboardsPage />) },
      { path: 'players', element: withSuspense(<PlayersPage />) },
      { path: 'players/:steamid', element: withSuspense(<PlayerDetailPage />) },
      { path: 'compare', element: withSuspense(<ComparePage />) },
      { path: 'maps', element: withSuspense(<MapsPage />) },
      { path: 'maps/:slug', element: withSuspense(<MapDetailPage />) },
      { path: 'records', element: withSuspense(<RecordsPage />) },
      { path: 'servers', element: withSuspense(<ServersPage />) },
      { path: 'points', element: withSuspense(<PointsLeaderboardsPage />) },
      { path: 'players/:steamid/points', element: withSuspense(<PlayerPointsPage />) },
      { path: '*', element: <NotFoundPage /> },
    ],
  },
]);

function withSuspense(element: ReactNode) {
  return <Suspense fallback={<PageLoading />}>{element}</Suspense>;
}
