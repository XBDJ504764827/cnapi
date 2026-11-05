import { ArrowLeft, Ban, Gauge, Timer, UserRound } from 'lucide-react';
import { Link, useParams } from 'react-router';

import { EmptyState } from '@/admin/components/common/empty-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { Avatar, AvatarFallback } from '@/admin/components/ui/avatar';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent } from '@/admin/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/admin/components/ui/tabs';

const plannedSections = [
  {
    value: 'profile',
    label: '玩家资料',
    icon: UserRound,
    title: '玩家资料接口未接入',
    description: '昵称、国家、Prime 状态、首次/最近进入时间等字段将在玩家详情接口中提供。',
  },
  {
    value: 'records',
    label: '成绩历史',
    icon: Timer,
    title: '成绩历史接口未接入',
    description: '将按地图、模式、TP/PRO 展示成绩、WR 标记与审核状态。',
  },
  {
    value: 'jumpstats',
    label: 'Jumpstats',
    icon: Gauge,
    title: 'Jumpstats 接口未接入',
    description: '将展示玩家 Jumpstats 最佳记录与分布。',
  },
  {
    value: 'bans',
    label: '封禁历史',
    icon: Ban,
    title: '封禁历史接口未接入',
    description: '将展示封禁类型、原因、执行者、生效时间与解封记录。',
  },
] as const;

/** Player detail shell. Tabs are interactive; every panel is an honest empty state. */
export function PlayerDetailPage() {
  const { playerId } = useParams<{ playerId: string }>();

  return (
    <div className="space-y-5">
      <Button variant="ghost" size="sm" asChild className="-ml-2 w-fit">
        <Link to="/admin/players">
          <ArrowLeft />
          返回玩家列表
        </Link>
      </Button>

      <PageHeader
        title="玩家详情"
        description="玩家档案、成绩、Jumpstats 与封禁历史的统一入口。"
        meta={
          <>
            <span className="font-mono">{playerId ?? '未知 SteamID64'}</span>
            <span className="text-border-strong">·</span>
            <Badge variant="outline">接口未接入</Badge>
          </>
        }
      />

      <Card>
        <CardContent className="flex flex-col gap-4 pt-5 sm:flex-row sm:items-center sm:justify-between">
          <div className="flex items-center gap-3">
            <Avatar className="size-12">
              <AvatarFallback>
                <UserRound className="size-5" />
              </AvatarFallback>
            </Avatar>
            <div className="space-y-1">
              <p className="text-base font-semibold">未知玩家</p>
              <p className="font-mono text-xs text-muted-foreground">{playerId ?? '—'}</p>
            </div>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant="outline">WR —</Badge>
            <Badge variant="outline">积分 —</Badge>
            <Badge variant="outline">最近活动 —</Badge>
          </div>
        </CardContent>
      </Card>

      <Callout
        variant="info"
        title="玩家详情接口尚未实现"
        description="下方标签页可以切换，但不会发起伪造请求，也不会展示模拟资料。"
      />

      <Tabs defaultValue="profile">
        <TabsList>
          {plannedSections.map((section) => (
            <TabsTrigger key={section.value} value={section.value}>
              <section.icon className="size-3.5" />
              {section.label}
            </TabsTrigger>
          ))}
        </TabsList>

        {plannedSections.map((section) => (
          <TabsContent key={section.value} value={section.value}>
            <EmptyState
              icon={section.icon}
              title={section.title}
              description={section.description}
              hint="数据来自 /api/v1 玩家相关接口（阶段 4）"
            />
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}
