import { BookOpen, ExternalLink, Info, LogOut, Settings, ShieldCheck } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/admin/components/ui/dropdown-menu';
import { Avatar, AvatarFallback, AvatarImage } from '@/admin/components/ui/avatar';
import { Button } from '@/admin/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogClose,
} from '@/admin/components/ui/dialog';
import { useSystemInfo } from '@/admin/hooks/use-system-info';
import { apiDocsUrl } from '@/admin/lib/env';
import { useLogout, useSession, userInitials } from '@/auth/session';

/** Account menu of the console: the real Steam identity, never a placeholder. */
export function UserMenu() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const infoQuery = useSystemInfo();
  const sessionQuery = useSession();
  const logout = useLogout();
  const user = sessionQuery.data?.user;

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm" className="rounded-full" aria-label="账号菜单">
            <Avatar className="size-6 border-border">
              {user?.avatar_url ? <AvatarImage src={user.avatar_url} alt={user.display_name} /> : null}
              <AvatarFallback className="bg-primary/15 text-[10px] font-semibold text-primary">
                {userInitials(user)}
              </AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-64">
          <DropdownMenuLabel className="normal-case">
            <span className="block truncate text-[13px] font-medium tracking-normal text-foreground">
              {user?.display_name ?? '未登录'}
            </span>
            <span className="block truncate font-mono text-[11px] font-normal tracking-normal text-muted-foreground">
              {user?.steam_id64 ?? '—'}
            </span>
            {user?.is_admin ? (
              <span className="mt-1 inline-flex items-center gap-1 text-[11px] font-normal tracking-normal text-primary">
                <ShieldCheck className="size-3" />
                管理员
              </span>
            ) : null}
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link to="/admin/system/settings">
              <Settings />
              系统设置
            </Link>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <a href="/">
              <BookOpen />
              公开门户
            </a>
          </DropdownMenuItem>
          <DropdownMenuItem asChild>
            <a href={apiDocsUrl} target="_blank" rel="noreferrer">
              <BookOpen />
              API 文档
              <ExternalLink className="ml-auto size-3 opacity-60" />
            </a>
          </DropdownMenuItem>
          <DropdownMenuSeparator />
          <DropdownMenuItem onSelect={() => setAboutOpen(true)}>
            <Info />
            关于 CNGOKZAPI
          </DropdownMenuItem>
          <DropdownMenuItem
            onSelect={(event) => {
              event.preventDefault();
              logout.mutate();
            }}
            disabled={logout.isPending}
          >
            <LogOut />
            {logout.isPending ? '正在退出…' : '退出登录'}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>

      <Dialog open={aboutOpen} onOpenChange={setAboutOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <div className="mb-1 flex size-9 items-center justify-center rounded-lg border border-primary/30 bg-primary/15 text-[11px] font-bold text-primary">
              KZ
            </div>
            <DialogTitle>CNGOKZAPI</DialogTitle>
            <DialogDescription>
              面向 CS:GO / CS2 KZ 社区的数据管理与 API 平台。管理后台仅对持有 admin 角色的 Steam 账号开放。
            </DialogDescription>
          </DialogHeader>

          <dl className="mt-4 space-y-2 rounded-lg border border-border bg-surface-elevated/40 px-3.5 py-3 text-xs">
            <div className="flex items-center justify-between gap-4">
              <dt className="text-muted-foreground">API 版本</dt>
              <dd className="font-mono">{infoQuery.data ? `/${infoQuery.data.api_version}` : '—'}</dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-muted-foreground">后端版本</dt>
              <dd className="font-mono">{infoQuery.data?.version ?? '—'}</dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-muted-foreground">运行环境</dt>
              <dd className="font-mono">{infoQuery.data?.environment ?? '—'}</dd>
            </div>
            <div className="flex items-center justify-between gap-4">
              <dt className="text-muted-foreground">Steam 账号</dt>
              <dd className="truncate font-mono">{user?.steam_id64 ?? '—'}</dd>
            </div>
          </dl>

          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline" size="sm">
                关闭
              </Button>
            </DialogClose>
            <Button size="sm" asChild>
              <Link to="/admin/developer/api-docs" onClick={() => setAboutOpen(false)}>
                查看 API 文档
              </Link>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
