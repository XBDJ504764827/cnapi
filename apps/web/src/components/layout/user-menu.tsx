import { BookOpen, ExternalLink, Info, Settings } from 'lucide-react';
import { useState } from 'react';
import { Link } from 'react-router';

import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Avatar, AvatarFallback } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogClose,
} from '@/components/ui/dialog';
import { useSystemInfo } from '@/hooks/use-system-info';
import { apiDocsUrl } from '@/lib/env';

/** Account menu. Authentication is not implemented yet, so no fake logout. */
export function UserMenu() {
  const [aboutOpen, setAboutOpen] = useState(false);
  const infoQuery = useSystemInfo();

  return (
    <>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm" className="rounded-full" aria-label="账号菜单">
            <Avatar className="size-6 border-border">
              <AvatarFallback className="bg-primary/15 text-[10px] font-semibold text-primary">KZ</AvatarFallback>
            </Avatar>
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-56">
          <DropdownMenuLabel className="normal-case">
            <span className="block text-[13px] font-medium tracking-normal text-foreground">本地管理员</span>
            <span className="block text-[11px] font-normal tracking-normal text-muted-foreground">
              认证系统尚未启用
            </span>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuItem asChild>
            <Link to="/system/settings">
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
              面向 CS:GO / CS2 KZ 社区的数据管理与 API 平台。当前为第三阶段：网站 UI/UX 设计与前端视觉重构。
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
          </dl>

          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline" size="sm">
                关闭
              </Button>
            </DialogClose>
            <Button size="sm" asChild>
              <Link to="/developer/api-docs" onClick={() => setAboutOpen(false)}>
                查看 API 文档
              </Link>
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
