import { Activity, ArrowLeft, Gauge, Server as ServerIcon, Timer } from 'lucide-react';
import { Link, useParams } from 'react-router';

import { EmptyState } from '@/components/common/empty-state';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { Card, CardContent } from '@/components/ui/card';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

const plannedSections = [
  {
    value: 'overview',
    label: '概览',
    icon: ServerIcon,
    title: '服务器概览接口未接入',
    description: '将展示名称、地址、所有者、当前地图、插件版本与在线状态。',
  },
  {
    value: 'records',
    label: '成绩提交',
    icon: Timer,
    title: '提交记录接口未接入',
    description: '将展示该服务器提交的成绩数量、最近提交与审核状态。',
  },
  {
    value: 'jumpstats',
    label: 'Jumpstats',
    icon: Gauge,
    title: 'Jumpstats 接口未接入',
    description: '将展示该服务器的 Jumpstats 提交情况。',
  },
  {
    value: 'heartbeat',
    label: '心跳',
    icon: Activity,
    title: '心跳历史接口未接入',
    description: '将展示最近心跳、离线时长与插件版本变化。',
  },
] as const;

/** Server detail shell with interactive tabs and honest empty states. */
export function ServerDetailPage() {
  const { serverId } = useParams<{ serverId: string }>();

  return (
    <div className="space-y-5">
      <Button variant="ghost" size="sm" asChild className="-ml-2 w-fit">
        <Link to="/servers">
          <ArrowLeft />
          返回服务器列表
        </Link>
      </Button>

      <PageHeader
        title="服务器详情"
        description="服务器状态、提交记录与插件心跳。"
        meta={
          <>
            <span className="font-mono">#{serverId ?? '—'}</span>
            <span className="text-border-strong">·</span>
            <Badge variant="outline">接口未接入</Badge>
          </>
        }
      />

      <Card>
        <CardContent className="flex flex-wrap items-center justify-between gap-4 pt-5">
          <div className="space-y-1">
            <p className="text-base font-semibold">未知服务器</p>
            <p className="font-mono text-xs text-muted-foreground">地址 — · 插件 —</p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Badge variant="outline">状态 —</Badge>
            <Badge variant="outline">当前地图 —</Badge>
            <Badge variant="outline">在线玩家 —</Badge>
          </div>
        </CardContent>
      </Card>

      <Callout
        variant="info"
        title="服务器详情接口尚未实现"
        description="标签页可切换；在接口接入前不会展示模拟心跳、CPU 或内存数据。"
      />

      <Tabs defaultValue="overview">
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
              hint="数据来自 /api/v1/servers（阶段 4）"
            />
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}
