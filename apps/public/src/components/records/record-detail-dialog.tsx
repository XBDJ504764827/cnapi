import { ExternalLink, Film, Trophy } from 'lucide-react';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router';

import { ErrorState } from '@/components/common/error-state';
import { ModeBadge } from '@/components/common/mode-badge';
import { PlayerAvatar, PlayerNameLink } from '@/components/common/player-avatar';
import { TierBadge } from '@/components/common/tier-badge';
import { TimeTypeBadge } from '@/components/common/time-type-badge';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '@/components/ui/dialog';
import { Skeleton } from '@/components/ui/skeleton';
import { formatDateTime, formatSeconds } from '@/lib/format';
import { publicQueries } from '@/services/public-service';
import type { RecordSummaryPayload } from '@cngokz/shared-types';

export function RecordDetailDialog({
  record,
  onOpenChange,
}: {
  record: RecordSummaryPayload | null;
  onOpenChange: (open: boolean) => void;
}) {
  const query = useQuery({ ...publicQueries.record(record?.id ?? 0), enabled: record !== null });

  return (
    <Dialog open={record !== null} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>成绩详情</DialogTitle>
          <DialogDescription>完成时间指跑图时长；上传时间为记录进入数据库的时间。</DialogDescription>
        </DialogHeader>

        {query.isLoading ? (
          <div className="mt-4 space-y-2">
            <Skeleton className="h-5 w-40" />
            <Skeleton className="h-20 w-full" />
          </div>
        ) : query.isError ? (
          <div className="mt-4">
            <ErrorState
              title="无法加载成绩详情"
              description={query.error.message}
              onRetry={() => void query.refetch()}
            />
          </div>
        ) : query.data ? (
          <div className="mt-4 space-y-4">
            <div className="flex items-center justify-between gap-3 rounded-lg border border-border bg-surface-elevated/40 px-3.5 py-3">
              <div className="flex items-center gap-2.5">
                <PlayerAvatar
                  steamId64={query.data.record.player.steam_id64}
                  displayName={query.data.record.player.display_name}
                  size="md"
                />
                <div>
                  <PlayerNameLink
                    steamId64={query.data.record.player.steam_id64}
                    displayName={query.data.record.player.display_name}
                  />
                  <p className="font-mono text-[10px] text-muted-foreground">{query.data.record.player.steam_id64}</p>
                </div>
              </div>
              <div className="text-right">
                <p className="font-mono text-xl font-semibold tabular-nums">
                  {formatSeconds(query.data.record.time_seconds)}s
                </p>
                <p className="text-[11px] text-muted-foreground">全球 #{query.data.record.rank}</p>
              </div>
            </div>

            <dl className="grid grid-cols-2 gap-x-4 gap-y-2 text-xs">
              <div>
                <dt className="text-muted-foreground">地图</dt>
                <dd className="mt-0.5">
                  <Link
                    to={`/maps/${query.data.record.map.slug}`}
                    className="font-mono hover:text-primary hover:underline"
                  >
                    {query.data.record.map.name}
                  </Link>
                  <span className="ml-1 text-muted-foreground">
                    · {query.data.record.course.stage === 0 ? 'Main' : `Bonus ${query.data.record.course.stage}`}
                  </span>
                </dd>
              </div>
              <div>
                <dt className="text-muted-foreground">模式 / 类型</dt>
                <dd className="mt-1 flex items-center gap-1.5">
                  <ModeBadge code={query.data.record.mode.code} />
                  <TimeTypeBadge value={query.data.record.time_type} />
                  <TierBadge tier={query.data.record.tier} />
                </dd>
              </div>
              <div>
                <dt className="text-muted-foreground">上传时间</dt>
                <dd className="mt-0.5">{formatDateTime(query.data.record.created_at)}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">来源服务器</dt>
                <dd className="mt-0.5 truncate">{query.data.server?.name ?? '未记录'}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">传送次数</dt>
                <dd className="mt-0.5 tabular-nums">{query.data.record.teleports}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">世界纪录</dt>
                <dd className="mt-1">
                  {query.data.record.is_world_record ? (
                    <Badge variant="primary">
                      <Trophy className="size-3" />
                      当前 WR
                    </Badge>
                  ) : (
                    <span className="text-muted-foreground">否</span>
                  )}
                </dd>
              </div>
            </dl>

            {query.data.replay_id ? (
              <Callout
                variant="neutral"
                icon={Film}
                title="Replay 可用"
                description="Replay 文件已上传到对象存储；公开下载接口尚未开放。"
                action={
                  <Button variant="outline" size="xs" disabled title="公开 Replay 下载接口尚未实现">
                    下载
                    <ExternalLink className="size-3" />
                  </Button>
                }
              />
            ) : null}
          </div>
        ) : null}
      </DialogContent>
    </Dialog>
  );
}
