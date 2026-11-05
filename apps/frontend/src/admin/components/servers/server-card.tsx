import { Activity, ArrowRight, Server as ServerIcon } from 'lucide-react';
import { Link } from 'react-router';

import { Badge } from '@/admin/components/ui/badge';
import { Card, CardContent, CardFooter, CardHeader } from '@/admin/components/ui/card';
import { StatusIndicator } from '@/admin/components/ui/status-indicator';
import type { StatusTone } from '@/admin/types';

export interface ServerCardData {
  id: number;
  name: string;
  address: string;
  tone: StatusTone;
  currentMap: string | null;
  players: number | null;
  pluginVersion: string | null;
  lastHeartbeat: string | null;
}

/** Card representation of a server, used by the card view on the servers page. */
export function ServerCard({ server }: { server: ServerCardData }) {
  return (
    <Card className="gap-0">
      <CardHeader className="flex-row items-start justify-between gap-3">
        <div className="flex min-w-0 items-center gap-2.5">
          <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
            <ServerIcon className="size-4" aria-hidden="true" />
          </span>
          <div className="min-w-0">
            <p className="truncate text-[13px] font-medium">{server.name}</p>
            <p className="truncate font-mono text-[11px] text-muted-foreground">{server.address}</p>
          </div>
        </div>
        <StatusIndicator tone={server.tone} />
      </CardHeader>
      <CardContent className="space-y-2 pt-3">
        <div className="flex items-center justify-between gap-3 text-xs">
          <span className="text-muted-foreground">当前地图</span>
          <span className="truncate font-mono">{server.currentMap ?? '—'}</span>
        </div>
        <div className="flex items-center justify-between gap-3 text-xs">
          <span className="text-muted-foreground">在线玩家</span>
          <span className="tabular-nums">{server.players ?? '—'}</span>
        </div>
        <div className="flex items-center justify-between gap-3 text-xs">
          <span className="text-muted-foreground">插件版本</span>
          <Badge variant="outline" className="max-w-36 truncate font-mono">
            {server.pluginVersion ?? '—'}
          </Badge>
        </div>
        <div className="flex items-center justify-between gap-3 text-xs">
          <span className="text-muted-foreground">最近心跳</span>
          <span className="inline-flex items-center gap-1 text-muted-foreground">
            <Activity className="size-3" aria-hidden="true" />
            {server.lastHeartbeat ?? '—'}
          </span>
        </div>
      </CardContent>
      <CardFooter className="justify-end">
        <Link
          to={`/servers/${server.id}`}
          className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
        >
          服务器详情
          <ArrowRight className="size-3" />
        </Link>
      </CardFooter>
    </Card>
  );
}
