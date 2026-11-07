import { LayoutDashboard, Loader2, LogIn, LogOut, User } from 'lucide-react';
import { Link } from 'react-router';

import { currentPath, startSteamLogin, useLogout, useSession, userInitials } from '@/auth/session';
import { Avatar, AvatarFallback, AvatarImage } from '@/public/components/ui/avatar';
import { Button } from '@/public/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/public/components/ui/dropdown-menu';

/**
 * Header account control. Steam OpenID is the only way in, so the anonymous
 * state is a single "Steam 登录" button and nothing else.
 */
export function SteamLogin() {
  const sessionQuery = useSession();
  const logout = useLogout();
  const session = sessionQuery.data;

  if (sessionQuery.isPending) {
    return (
      <span
        className="inline-flex h-8 w-8 items-center justify-center rounded-md border border-border/60"
        aria-hidden="true"
      >
        <Loader2 className="size-3.5 animate-spin text-muted-foreground" />
      </span>
    );
  }

  if (!session?.authenticated || !session.user) {
    return (
      <Button variant="outline" size="sm" onClick={() => startSteamLogin(currentPath())} aria-label="使用 Steam 登录">
        <LogIn />
        <span className="hidden sm:inline">Steam 登录</span>
      </Button>
    );
  }

  const user = session.user;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="sm" className="gap-2 pl-1" aria-label="账号菜单">
          <Avatar className="size-6 border border-border">
            {user.avatar_url ? <AvatarImage src={user.avatar_url} alt={user.display_name} /> : null}
            <AvatarFallback className="bg-primary/15 text-[10px] font-semibold text-primary">
              {userInitials(user)}
            </AvatarFallback>
          </Avatar>
          <span className="hidden max-w-28 truncate text-[13px] font-medium sm:inline">{user.display_name}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel className="normal-case">
          <span className="block truncate text-[13px] font-medium tracking-normal text-foreground">
            {user.display_name}
          </span>
          <span className="block font-mono text-[11px] font-normal tracking-normal text-muted-foreground">
            {user.steam_id64}
          </span>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        {user.is_admin ? (
          <DropdownMenuItem asChild>
            <Link to="/admin/dashboard">
              <LayoutDashboard />
              管理后台
            </Link>
          </DropdownMenuItem>
        ) : null}
        <DropdownMenuItem asChild>
          <Link to={`/players/${user.steam_id64}`}>
            <User />
            我的记录与排名
          </Link>
        </DropdownMenuItem>
        <DropdownMenuSeparator />
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
  );
}
