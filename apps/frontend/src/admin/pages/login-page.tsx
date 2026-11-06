import { ArrowLeft, ShieldCheck } from 'lucide-react';
import { Link, Navigate, useSearchParams } from 'react-router';

import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent } from '@/admin/components/ui/card';
import { authErrorMessage, readAuthError, startSteamLogin, useLogout, useSession } from '@/auth/session';

/**
 * Admin sign-in. Steam is the only identity provider: there is no password
 * form, and no account can be created from this page.
 */
export function LoginPage() {
  const [searchParams] = useSearchParams();
  const sessionQuery = useSession();
  const logout = useLogout();

  const requested = searchParams.get('next');
  // Only same-console paths are accepted, so `?next=` cannot become an open
  // redirect to another site.
  const next = requested && requested.startsWith('/admin') ? requested : '/admin/dashboard';
  const message = authErrorMessage(readAuthError());

  const user = sessionQuery.data?.user;

  // Already an administrator (for example after a second tab logged in).
  if (user?.is_admin) {
    return <Navigate to={next} replace />;
  }

  return (
    <div className="flex min-h-screen flex-col bg-background">
      <header className="flex h-14 items-center px-4 sm:px-6">
        <Link
          to="/"
          className="inline-flex items-center gap-2 rounded-md text-[13px] text-muted-foreground transition-colors outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/40"
        >
          <ArrowLeft className="size-3.5" />
          返回公开门户
        </Link>
        <span className="ml-auto text-[11px] tracking-wide text-muted-foreground uppercase">CNGOKZ Console</span>
      </header>

      <main className="flex flex-1 items-center justify-center px-4 py-10">
        <div className="w-full max-w-sm space-y-4">
          <Card>
            <CardContent className="space-y-5 p-6">
              <div className="flex items-center gap-3">
                <span className="flex size-9 items-center justify-center rounded-lg border border-primary/30 bg-primary/15 text-[11px] font-bold text-primary">
                  KZ
                </span>
                <div className="min-w-0">
                  <h1 className="text-[15px] leading-tight font-semibold tracking-tight">管理后台登录</h1>
                  <p className="text-[11px] text-muted-foreground">CNGOKZ · KZ DATA PLATFORM</p>
                </div>
              </div>

              <p className="text-[13px] leading-relaxed text-muted-foreground">
                使用 Steam 账号登录。登录只用于验证身份，我们不会获取你的 Steam 密码，也不会向你的账号发送任何内容。
              </p>

              {message ? <Callout variant="danger" title="登录未完成" description={message} /> : null}

              {user && !user.is_admin ? (
                <Callout
                  variant="warning"
                  title="当前账号不是管理员"
                  description={
                    <span>
                      已登录为 <span className="font-medium text-foreground">{user.display_name}</span>（
                      <span className="font-mono">{user.steam_id64}</span>），但没有 admin 角色。
                    </span>
                  }
                />
              ) : null}

              {sessionQuery.isError ? (
                <Callout variant="warning" title="无法读取登录状态" description="认证接口暂时不可用，请稍后重试。" />
              ) : null}

              <Button
                className="w-full"
                size="lg"
                onClick={() => startSteamLogin(next)}
                disabled={sessionQuery.isPending}
              >
                <SteamMark />
                使用 Steam 登录
              </Button>

              {user ? (
                <Button
                  className="w-full"
                  size="sm"
                  variant="ghost"
                  onClick={() => logout.mutate()}
                  disabled={logout.isPending}
                >
                  退出当前账号
                </Button>
              ) : null}
            </CardContent>
          </Card>

          <p className="flex items-start gap-1.5 px-1 text-[11px] leading-relaxed text-muted-foreground">
            <ShieldCheck className="mt-0.5 size-3.5 shrink-0" />
            管理后台仅限管理员访问。普通玩家可以正常浏览公开门户，但无法进入本控制台。
          </p>
        </div>
      </main>
    </div>
  );
}

/** Steam wordmark-free glyph: a simple circle/pipe mark drawn inline. */
function SteamMark() {
  return (
    <svg viewBox="0 0 24 24" aria-hidden="true" fill="currentColor" className="size-4">
      <path d="M11.98 2a10 10 0 0 0-9.98 9.16l5.36 2.22a2.83 2.83 0 0 1 1.62-.5h.16l2.39-3.46v-.05a3.78 3.78 0 1 1 3.78 3.78h-.09l-3.41 2.43v.13a2.84 2.84 0 0 1-5.62.6L2.2 14.6A10 10 0 1 0 11.98 2Zm-4.3 14.5 1.22.5a2.13 2.13 0 0 0 2.76-1.15 2.13 2.13 0 0 0-1.2-2.76l-1.32-.55a2.84 2.84 0 0 1 1.1 3.96Zm8.1-5.6a2.52 2.52 0 1 0 0-5.04 2.52 2.52 0 0 0 0 5.04Zm0-.79a1.73 1.73 0 1 1 0-3.46 1.73 1.73 0 0 1 0 3.46Z" />
    </svg>
  );
}
