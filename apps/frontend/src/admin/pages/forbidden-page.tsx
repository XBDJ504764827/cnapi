import { LayoutDashboard, LogOut, ShieldAlert, Home } from 'lucide-react';
import { Link } from 'react-router';

import { Button } from '@/admin/components/ui/button';
import { Card, CardContent } from '@/admin/components/ui/card';
import { startSteamLogin, useLogout, useSession } from '@/auth/session';

type Reason = 'not_admin' | 'unavailable';

/**
 * Shown instead of the console when the caller is authenticated but is not an
 * administrator (or the session endpoint cannot be reached).
 */
export function ForbiddenPage({ reason = 'not_admin' }: { reason?: Reason }) {
  const sessionQuery = useSession();
  const logout = useLogout();
  const user = sessionQuery.data?.user;

  return (
    <div className="flex min-h-screen items-center justify-center bg-background px-4 py-10">
      <Card className="w-full max-w-md">
        <CardContent className="space-y-5 p-6">
          <div className="flex size-10 items-center justify-center rounded-lg border border-warning/30 bg-warning/15 text-warning">
            <ShieldAlert className="size-5" />
          </div>

          <div className="space-y-1.5">
            <h1 className="text-lg font-semibold tracking-tight">
              {reason === 'not_admin' ? '没有管理后台访问权限' : '暂时无法验证登录状态'}
            </h1>
            <p className="text-[13px] leading-relaxed text-muted-foreground">
              {reason === 'not_admin'
                ? '管理后台仅对管理员开放。你的 Steam 账号已登录，但没有 admin 角色。如需权限，请联系站点管理员为你的 SteamID 授予角色。'
                : '无法连接认证接口，请稍后重试。如果问题持续存在，请联系站点管理员。'}
            </p>
          </div>

          {user ? (
            <dl className="space-y-1 rounded-lg border border-border bg-surface-elevated/40 px-3.5 py-3 text-xs">
              <div className="flex items-center justify-between gap-4">
                <dt className="text-muted-foreground">当前账号</dt>
                <dd className="truncate font-medium text-foreground">{user.display_name}</dd>
              </div>
              <div className="flex items-center justify-between gap-4">
                <dt className="text-muted-foreground">SteamID64</dt>
                <dd className="font-mono">{user.steam_id64}</dd>
              </div>
            </dl>
          ) : null}

          <div className="flex flex-wrap gap-2">
            <Button size="sm" asChild>
              <Link to="/">
                <Home />
                返回公开门户
              </Link>
            </Button>
            <Button size="sm" variant="outline" onClick={() => startSteamLogin('/admin/dashboard')}>
              <LayoutDashboard />
              换一个 Steam 账号
            </Button>
            {user ? (
              <Button size="sm" variant="ghost" onClick={() => logout.mutate()} disabled={logout.isPending}>
                <LogOut />
                退出登录
              </Button>
            ) : null}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
