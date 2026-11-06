import { BarChart3, Copy, KeyRound, Plus, ShieldAlert } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { PageHeader } from '@/admin/components/common/page-header';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/admin/components/ui/card';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogClose,
} from '@/admin/components/ui/dialog';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Input } from '@/admin/components/ui/input';
import { Label } from '@/admin/components/ui/label';
import { Pagination } from '@/admin/components/ui/pagination';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { ListPanel } from '@/admin/components/common/list-panel';
import { useTableState } from '@/admin/hooks/use-table-state';
import { pageSizeOptions } from '@/admin/lib/kz-constants';

interface ApiKeyRow {
  id: number;
  name: string;
  prefix: string;
  scopes: string;
  lastUsedAt: string | null;
  createdAt: string;
  status: 'active' | 'revoked';
}

const columns: DataTableColumn<ApiKeyRow>[] = [
  {
    id: 'name',
    header: '名称',
    cell: (row) => (
      <div className="flex items-center gap-2.5">
        <span className="flex size-7 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
          <KeyRound className="size-3.5" aria-hidden="true" />
        </span>
        <span className="text-[13px] font-medium">{row.name}</span>
      </div>
    ),
  },
  { id: 'prefix', header: '前缀', cell: (row) => <span className="font-mono text-xs">{row.prefix}…</span> },
  { id: 'scopes', header: '权限', cell: (row) => <Badge variant="outline">{row.scopes}</Badge> },
  {
    id: 'lastUsed',
    header: '最后使用',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.lastUsedAt ?? '从未使用'}</span>,
  },
  {
    id: 'createdAt',
    header: '创建时间',
    cell: (row) => <span className="text-xs text-muted-foreground">{row.createdAt}</span>,
  },
  {
    id: 'status',
    header: '状态',
    cell: (row) => (
      <Badge variant={row.status === 'active' ? 'success' : 'outline'}>
        {row.status === 'active' ? '有效' : '已吊销'}
      </Badge>
    ),
  },
  {
    id: 'actions',
    header: '',
    headerClassName: 'text-right',
    className: 'text-right',
    cell: () => (
      <Button variant="ghost" size="xs" disabled title="吊销接口尚未实现">
        吊销
      </Button>
    ),
  },
];

export function ApiKeysPage() {
  const table = useTableState();
  const [createOpen, setCreateOpen] = useState(false);
  const [keyName, setKeyName] = useState('');
  const [scope, setScope] = useState('submit');

  return (
    <div className="space-y-5">
      <PageHeader
        title="API 密钥"
        description="为游戏服务器与第三方插件签发访问凭据。密钥只保存哈希，明文仅在创建时展示一次。"
        actions={
          <Button size="sm" onClick={() => setCreateOpen(true)}>
            <Plus />
            创建 API Key
          </Button>
        }
      />

      <Callout
        variant="warning"
        icon={ShieldAlert}
        title="密钥服务尚未实现"
        description="后端尚未提供创建/吊销接口。本页展示完整交互与安全约束，但不会生成或展示任何真实密钥。"
      />

      <ListPanel
        footer={
          <Pagination
            page={table.page}
            pageSize={table.pageSize}
            total={0}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={pageSizeOptions}
          />
        }
      >
        <DataTable
          columns={columns}
          rows={[]}
          getRowKey={(row) => row.id}
          minWidthClass="min-w-[900px]"
          empty={
            <EmptyState
              icon={KeyRound}
              title="暂无 API 密钥"
              description="密钥创建接口尚未实现。接入后，创建成功时明文只显示一次，后端仅保存哈希与前缀。"
            />
          }
        />
      </ListPanel>

      <Card>
        <CardHeader>
          <CardTitle>API 请求统计</CardTitle>
          <CardDescription>按密钥统计调用量、错误率与最后使用时间</CardDescription>
        </CardHeader>
        <CardContent>
          <EmptyState
            icon={BarChart3}
            title="暂无请求统计"
            description="请求日志与聚合指标接口尚未实现，当前不展示模拟调用量。"
            className="min-h-40"
          />
        </CardContent>
      </Card>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>创建 API Key</DialogTitle>
            <DialogDescription>
              密钥用于游戏服务器或第三方插件调用 CNGOKZAPI。明文只在创建成功时展示一次，后端只保存哈希。
            </DialogDescription>
          </DialogHeader>

          <Callout
            variant="warning"
            title="创建接口尚未接入"
            description="填写内容不会发送到后端。接入后将显示一次性密钥，并提供复制与吊销操作。"
            className="mt-4"
          />

          <div className="mt-4 space-y-3">
            <div className="space-y-1.5">
              <Label htmlFor="api-key-name">名称</Label>
              <Input
                id="api-key-name"
                value={keyName}
                onChange={(event) => setKeyName(event.target.value)}
                placeholder="例如：KZ 服务器 #1"
                autoComplete="off"
              />
            </div>
            <div className="space-y-1.5">
              <Label htmlFor="api-key-scope">权限范围</Label>
              <Select value={scope} onValueChange={setScope}>
                <SelectTrigger id="api-key-scope">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="submit">成绩与 Jumpstats 提交</SelectItem>
                  <SelectItem value="heartbeat">服务器心跳</SelectItem>
                  <SelectItem value="read">只读查询</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-[11px] text-muted-foreground">权限模型将在认证阶段最终确定，选项仅供界面演示。</p>
            </div>
          </div>

          <DialogFooter>
            <DialogClose asChild>
              <Button variant="outline" size="sm">
                取消
              </Button>
            </DialogClose>
            <Button size="sm" variant="secondary" disabled title="后端接口尚未实现">
              <Copy />
              创建（接口未接入）
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
