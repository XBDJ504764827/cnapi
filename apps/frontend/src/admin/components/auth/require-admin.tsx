import { Navigate, Outlet, useLocation } from 'react-router';

import { PageLoading } from '@/admin/components/common/page-loading';
import { ForbiddenPage } from '@/admin/pages/forbidden-page';
import { useSession } from '@/auth/session';

/**
 * Route guard for the whole `/admin` tree.
 *
 * Three states are possible and each one gets a different answer:
 *   * anonymous → redirect to the login page, remembering where the user wanted
 *     to go (`?next=`), so the Steam round trip lands back on the same page;
 *   * authenticated without the `admin` role → 403, never the console shell;
 *   * authenticated administrator → render the console.
 *
 * The API enforces the same rule on every `/api/v1/admin/*` request, so this is
 * a UX guard, not the security boundary.
 */
export function RequireAdmin() {
  const location = useLocation();
  const sessionQuery = useSession();

  if (sessionQuery.isPending) {
    return (
      <div className="mx-auto w-full max-w-[1500px] px-4 py-8 sm:px-6 lg:px-8">
        <PageLoading label="正在验证登录状态" />
      </div>
    );
  }

  if (sessionQuery.isError) {
    return <ForbiddenPage reason="unavailable" />;
  }

  const session = sessionQuery.data;

  if (!session?.authenticated) {
    const next = encodeURIComponent(`${location.pathname}${location.search}`);
    return <Navigate to={`/admin/login?next=${next}`} replace />;
  }

  if (!session.user?.is_admin) {
    return <ForbiddenPage reason="not_admin" />;
  }

  return <Outlet />;
}
