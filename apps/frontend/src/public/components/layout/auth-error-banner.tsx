import { RefreshCw, X } from 'lucide-react';
import { useEffect, useState } from 'react';

import { authErrorMessage, readAuthError, startSteamLogin } from '@/auth/session';
import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';

/**
 * 登录失败时曾完全没有任何提示，用户只能看到右上角仍然是「Steam 登录」，
 * 无法判断是没登录成功还是页面没刷新。
 *
 * 这里把回调带回来的 `?auth_error=` 展示出来，并立刻把它从地址栏移除：
 * 否则 `currentPath()` 会把它带进下一次登录的 `redirect`（实测会出现
 * `redirect=/?auth_error=invalid_assertion`）。
 */
export function AuthErrorBanner() {
  const [code, setCode] = useState<string | null>(() => readAuthError());

  useEffect(() => {
    if (!code) return;
    const url = new URL(window.location.href);
    url.searchParams.delete('auth_error');
    window.history.replaceState({}, '', `${url.pathname}${url.search}${url.hash}`);
  }, [code]);

  if (!code) return null;
  const message = authErrorMessage(code);
  if (!message) return null;

  return (
    <div className="border-b border-danger/20 bg-danger/5">
      <div className="mx-auto w-full max-w-7xl px-4 py-2.5 sm:px-6 lg:px-8">
        <Callout
          variant="danger"
          title="Steam 登录未完成"
          description={message}
          action={
            <div className="flex items-center gap-1.5">
              <Button size="xs" variant="outline" onClick={() => startSteamLogin()}>
                <RefreshCw />
                重试
              </Button>
              <Button size="icon-xs" variant="ghost" onClick={() => setCode(null)} aria-label="关闭提示">
                <X />
              </Button>
            </div>
          }
        />
      </div>
    </div>
  );
}
