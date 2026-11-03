import type { ApiClientError } from '@cngokz/api-client';
import { ArrowRight, BookOpen, ExternalLink, Monitor, Moon, ShieldAlert, Sun } from 'lucide-react';
import { Link } from 'react-router';

import { Callout } from '@/components/ui/callout';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Switch } from '@/components/ui/switch';
import { useSystemInfo } from '@/hooks/use-system-info';
import { useTheme } from '@/hooks/use-theme';
import { apiDocsUrl, openApiDocumentUrl } from '@/lib/env';
import { formatDateTime, formatUptime } from '@/lib/format';
import { cn } from '@/lib/utils';
import { useUiStore, type Theme } from '@/stores/ui-store';

const themeOptions: Array<{ value: Theme; label: string; description: string; icon: typeof Sun }> = [
  { value: 'dark', label: '深色', description: '默认主题，深灰背景与冷色强调', icon: Moon },
  { value: 'light', label: '浅色', description: '明亮界面，适合强光环境', icon: Sun },
  { value: 'system', label: '跟随系统', description: '随操作系统外观自动切换', icon: Monitor },
];

export function SettingsPage() {
  const { theme, setTheme } = useTheme();
  const sidebarCollapsed = useUiStore((state) => state.sidebarCollapsed);
  const setSidebarCollapsed = useUiStore((state) => state.setSidebarCollapsed);
  const infoQuery = useSystemInfo();
  const infoError = infoQuery.error as ApiClientError | null;

  return (
    <div className="space-y-5">
      <PageHeader title="系统设置" description="外观、界面偏好与平台信息。设置保存在本地浏览器中，不需要后端接口。" />

      <div className="grid gap-4 lg:grid-cols-3">
        <div className="space-y-4 lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>外观</CardTitle>
              <CardDescription>主题会立即生效，并在刷新后保持。默认使用深色。</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid gap-2 sm:grid-cols-3" role="radiogroup" aria-label="主题">
                {themeOptions.map((option) => {
                  const Icon = option.icon;
                  const selected = theme === option.value;
                  return (
                    <button
                      key={option.value}
                      type="button"
                      role="radio"
                      aria-checked={selected}
                      onClick={() => setTheme(option.value)}
                      className={cn(
                        'flex flex-col gap-2 rounded-lg border px-3.5 py-3 text-left transition-colors',
                        selected
                          ? 'border-primary/50 bg-primary/8 ring-1 ring-primary/20'
                          : 'border-border bg-surface-elevated/30 hover:bg-surface-hover',
                      )}
                    >
                      <span className="flex items-center justify-between">
                        <Icon className={cn('size-4', selected ? 'text-primary' : 'text-muted-foreground')} />
                        {selected ? <Badge variant="primary">当前</Badge> : null}
                      </span>
                      <span className="text-[13px] font-medium">{option.label}</span>
                      <span className="text-[11px] leading-relaxed text-muted-foreground">{option.description}</span>
                    </button>
                  );
                })}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>界面</CardTitle>
              <CardDescription>导航与列表的显示偏好</CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <div className="flex items-center justify-between gap-4 rounded-md border border-border/70 bg-surface-elevated/30 px-3.5 py-3">
                <div className="space-y-0.5">
                  <p className="text-[13px] font-medium">折叠侧边栏</p>
                  <p className="text-[11px] text-muted-foreground">仅显示图标，悬停可查看导航名称。桌面端生效。</p>
                </div>
                <Switch checked={sidebarCollapsed} onCheckedChange={setSidebarCollapsed} aria-label="折叠侧边栏" />
              </div>
              <div className="flex items-center justify-between gap-4 rounded-md border border-border/70 bg-surface-elevated/30 px-3.5 py-3">
                <div className="space-y-0.5">
                  <p className="text-[13px] font-medium">页面切换动画</p>
                  <p className="text-[11px] text-muted-foreground">遵循系统“减少动态效果”设置，由 CSS 自动处理。</p>
                </div>
                <Badge variant="outline">自动</Badge>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>平台信息</CardTitle>
              <CardDescription>来自 /api/v1/system/info 的实时数据</CardDescription>
            </CardHeader>
            <CardContent>
              {infoQuery.isError ? (
                <Callout
                  variant="danger"
                  title="无法读取平台信息"
                  description={infoError?.message ?? '后端服务不可达。'}
                  action={
                    <Button variant="outline" size="xs" onClick={() => void infoQuery.refetch()}>
                      重试
                    </Button>
                  }
                />
              ) : (
                <dl className="grid gap-x-6 gap-y-3 text-[13px] sm:grid-cols-2">
                  <div className="flex items-baseline gap-2 border-b border-border/60 pb-2">
                    <dt className="w-20 shrink-0 text-muted-foreground">后端版本</dt>
                    <dd className="font-mono">
                      {infoQuery.isLoading ? <Skeleton className="h-4 w-16" /> : (infoQuery.data?.version ?? '—')}
                    </dd>
                  </div>
                  <div className="flex items-baseline gap-2 border-b border-border/60 pb-2">
                    <dt className="w-20 shrink-0 text-muted-foreground">运行环境</dt>
                    <dd className="font-mono">
                      {infoQuery.isLoading ? <Skeleton className="h-4 w-20" /> : (infoQuery.data?.environment ?? '—')}
                    </dd>
                  </div>
                  <div className="flex items-baseline gap-2 border-b border-border/60 pb-2">
                    <dt className="w-20 shrink-0 text-muted-foreground">API 版本</dt>
                    <dd className="font-mono">{infoQuery.data ? `/${infoQuery.data.api_version}` : '—'}</dd>
                  </div>
                  <div className="flex items-baseline gap-2 border-b border-border/60 pb-2">
                    <dt className="w-20 shrink-0 text-muted-foreground">运行时长</dt>
                    <dd className="font-mono">{infoQuery.data ? formatUptime(infoQuery.data.uptime_seconds) : '—'}</dd>
                  </div>
                  <div className="flex items-baseline gap-2 sm:col-span-2">
                    <dt className="w-20 shrink-0 text-muted-foreground">启动时间</dt>
                    <dd className="font-mono text-xs">{formatDateTime(infoQuery.data?.started_at)}</dd>
                  </div>
                </dl>
              )}
            </CardContent>
          </Card>
        </div>

        <div className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>快捷入口</CardTitle>
              <CardDescription>常用开发与运维页面</CardDescription>
            </CardHeader>
            <CardContent className="space-y-1">
              <SettingsLink to="/system/status" label="系统状态" description="依赖探测与健康检查" />
              <SettingsLink to="/developer/api-docs" label="API 文档" description="Swagger UI 与 OpenAPI" />
              <SettingsLink to="/developer/api-keys" label="API 密钥" description="服务器访问凭据" />
              <a
                href={apiDocsUrl}
                target="_blank"
                rel="noreferrer"
                className="flex items-center justify-between gap-3 rounded-md px-2 py-2 text-[13px] text-muted-foreground transition-colors hover:bg-surface-hover hover:text-foreground"
              >
                <span>
                  <span className="block font-medium text-foreground">Swagger UI</span>
                  <span className="block text-[11px]">外部打开后端文档</span>
                </span>
                <ExternalLink className="size-3.5" />
              </a>
              <a
                href={openApiDocumentUrl}
                target="_blank"
                rel="noreferrer"
                className="flex items-center justify-between gap-3 rounded-md px-2 py-2 text-[13px] text-muted-foreground transition-colors hover:bg-surface-hover hover:text-foreground"
              >
                <span>
                  <span className="block font-medium text-foreground">OpenAPI JSON</span>
                  <span className="block text-[11px]">下载接口契约</span>
                </span>
                <BookOpen className="size-3.5" />
              </a>
            </CardContent>
          </Card>

          <Callout
            variant="warning"
            icon={ShieldAlert}
            title="认证尚未启用"
            description="当前没有登录、角色或 API Key 校验。系统设置仅保存本地界面偏好，不会修改后端配置。"
          />

          <Card>
            <CardHeader>
              <CardTitle>关于</CardTitle>
              <CardDescription>第三阶段 · UI/UX 重构</CardDescription>
            </CardHeader>
            <CardContent className="space-y-2 text-xs text-muted-foreground">
              <p>CNGOKZAPI · CS:GO / CS2 KZ 数据管理与 API 平台</p>
              <p>设计体系：Inter、深灰表面、冷色强调、统一 8px 圆角与语义状态色。</p>
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

function SettingsLink({ to, label, description }: { to: string; label: string; description: string }) {
  return (
    <Link
      to={to}
      className="flex items-center justify-between gap-3 rounded-md px-2 py-2 transition-colors hover:bg-surface-hover"
    >
      <span>
        <span className="block text-[13px] font-medium text-foreground">{label}</span>
        <span className="block text-[11px] text-muted-foreground">{description}</span>
      </span>
      <ArrowRight className="size-3.5 text-muted-foreground" />
    </Link>
  );
}
