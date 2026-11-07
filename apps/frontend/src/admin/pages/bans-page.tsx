import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { Ban, FilterX, ShieldAlert, ShieldCheck } from 'lucide-react';
import { useState } from 'react';

import { EmptyState } from '@/admin/components/common/empty-state';
import { FilterBar, ListPanel } from '@/admin/components/common/list-panel';
import { PageHeader } from '@/admin/components/common/page-header';
import { PlayerCell } from '@/admin/components/common/player-cell';
import { Badge } from '@/admin/components/ui/badge';
import { Button } from '@/admin/components/ui/button';
import { Callout } from '@/admin/components/ui/callout';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/admin/components/ui/dialog';
import { DataTable, type DataTableColumn } from '@/admin/components/ui/data-table';
import { Input } from '@/admin/components/ui/input';
import { Label } from '@/admin/components/ui/label';
import { Pagination } from '@/admin/components/ui/pagination';
import { SearchInput } from '@/admin/components/ui/search-input';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/admin/components/ui/select';
import { api } from '@/admin/lib/api';
import { formatDateTime } from '@/admin/lib/format';
import { pageSizeOptions } from '@/admin/lib/kz-constants';
import { useDebouncedValue } from '@/admin/hooks/use-debounced-value';
import { useTableState } from '@/admin/hooks/use-table-state';
import type { components } from '@cngokz/shared-types';

type BanRow = components['schemas']['BanSummary'];

const statusMeta: Record<string, { label: string; variant: 'danger' | 'warning' | 'outline' }> = {
  active: { label: '生效中', variant: 'danger' },
  expired: { label: '已过期', variant: 'warning' },
  revoked: { label: '已撤销', variant: 'outline' },
};

const sourceLabel: Record<string, string> = {
  admin: '管理员',
  server: '服务器',
  globalapi_import: 'GlobalAPI 导入',
};

/** 管理端封禁控制台：读取 /api/v1/admin/bans，创建与撤销走同一鉴权通道。 */
export function BansPage() {
  const table = useTableState();
  const queryClient = useQueryClient();
  const [status, setStatus] = useState('all');
  const [createOpen, setCreateOpen] = useState(false);
  const [revokeTarget, setRevokeTarget] = useState<BanRow | null>(null);
  const [message, setMessage] = useState('');
  const search = useDebouncedValue(table.search);

  const listQuery = useQuery({
    queryKey: ['admin', 'bans', search, status, table.page, table.pageSize],
    queryFn: () =>
      api.bans.list({
        search: search || undefined,
        status,
        page: table.page,
        page_size: table.pageSize,
      }),
  });

  const invalidate = () => queryClient.invalidateQueries({ queryKey: ['admin', 'bans'] });

  const rows = listQuery.data?.items ?? [];
  const total = listQuery.data?.total ?? 0;

  const columns: DataTableColumn<BanRow>[] = [
    {
      id: 'player',
      header: '玩家',
      cell: (row) => (
        <PlayerCell
          steamId64={row.steam_id64}
          displayName={row.display_name}
          avatarUrl={row.avatar_url}
          linked={false}
        />
      ),
    },
    {
      id: 'type',
      header: '类型',
      cell: (row) => <Badge variant="outline">{row.ban_type}</Badge>,
    },
    {
      id: 'reason',
      header: '原因',
      cell: (row) => <span className="line-clamp-1 text-[13px]">{row.reason}</span>,
    },
    {
      id: 'issuedAt',
      header: '开始时间',
      cell: (row) => <span className="text-xs text-muted-foreground">{formatDateTime(row.issued_at)}</span>,
    },
    {
      id: 'expiresAt',
      header: '结束时间',
      cell: (row) => (
        <span className="text-xs text-muted-foreground">
          {row.expires_at ? formatDateTime(row.expires_at) : '永久'}
        </span>
      ),
    },
    {
      id: 'status',
      header: '状态',
      cell: (row) => {
        const meta = statusMeta[row.status] ?? statusMeta.revoked!;
        return <Badge variant={meta.variant}>{meta.label}</Badge>;
      },
    },
    {
      id: 'source',
      header: '来源',
      cell: (row) => <span className="text-xs text-muted-foreground">{sourceLabel[row.source] ?? row.source}</span>,
    },
    {
      id: 'actions',
      header: '',
      cell: (row) =>
        row.status === 'active' ? (
          <Button variant="ghost" size="xs" onClick={() => setRevokeTarget(row)}>
            <ShieldCheck />
            解封
          </Button>
        ) : (
          <span className="text-xs text-muted-foreground">
            {row.revoke_reason ? `已解封：${row.revoke_reason}` : '—'}
          </span>
        ),
    },
  ];

  return (
    <div className="space-y-5">
      <PageHeader
        title="封禁记录"
        description="封禁与解封审计。记录为追加式，不会物理删除；过期状态由 expires_at 推导。"
        meta={<span>共 {total} 条记录</span>}
        actions={
          <Button size="sm" onClick={() => setCreateOpen(true)}>
            <Ban />
            新建封禁
          </Button>
        }
      />

      {message ? <Callout variant="danger" title="操作失败" description={message} /> : null}
      {listQuery.isError ? (
        <Callout
          variant="danger"
          title="封禁列表读取失败"
          description={(listQuery.error as Error)?.message ?? '请重试'}
        />
      ) : null}

      <ListPanel
        toolbar={
          <FilterBar
            actions={
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  table.resetFilters();
                  setStatus('all');
                }}
              >
                <FilterX />
                重置
              </Button>
            }
          >
            <SearchInput
              value={table.search}
              onChange={table.setSearch}
              placeholder="搜索玩家或 SteamID64"
              className="w-full sm:w-64"
              aria-label="搜索封禁"
            />
            <Select
              value={status}
              onValueChange={(value) => {
                setStatus(value);
                table.setPage(1);
              }}
            >
              <SelectTrigger className="w-28" aria-label="封禁状态">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">全部状态</SelectItem>
                <SelectItem value="active">生效中</SelectItem>
                <SelectItem value="expired">已过期</SelectItem>
                <SelectItem value="revoked">已撤销</SelectItem>
              </SelectContent>
            </Select>
          </FilterBar>
        }
        footer={
          <Pagination
            page={table.page}
            pageSize={table.pageSize}
            total={total}
            onPageChange={table.setPage}
            onPageSizeChange={table.setPageSize}
            pageSizeOptions={pageSizeOptions}
          />
        }
      >
        <DataTable
          columns={columns}
          rows={rows}
          getRowKey={(row) => row.id}
          isLoading={listQuery.isLoading}
          empty={<EmptyState icon={ShieldAlert} title="暂无封禁记录" description="调整筛选条件，或新建一条封禁。" />}
        />
      </ListPanel>

      <CreateBanDialog
        open={createOpen}
        onOpenChange={setCreateOpen}
        onError={setMessage}
        onSuccess={async () => {
          setMessage('');
          await invalidate();
        }}
      />
      <RevokeBanDialog
        ban={revokeTarget}
        onClose={() => setRevokeTarget(null)}
        onError={setMessage}
        onSuccess={async () => {
          setMessage('');
          await invalidate();
        }}
      />
    </div>
  );
}

function CreateBanDialog({
  open,
  onOpenChange,
  onError,
  onSuccess,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onError: (message: string) => void;
  onSuccess: () => void | Promise<void>;
}) {
  const [steamId64, setSteamId64] = useState('');
  const [banType, setBanType] = useState('global');
  const [reason, setReason] = useState('');
  const [expiresAt, setExpiresAt] = useState('');

  const mutation = useMutation({
    mutationFn: () =>
      api.bans.create({
        steam_id64: steamId64.trim(),
        ban_type: banType,
        reason: reason.trim(),
        expires_at: expiresAt ? new Date(expiresAt).toISOString() : null,
      }),
    onSuccess: async () => {
      setSteamId64('');
      setReason('');
      setExpiresAt('');
      onOpenChange(false);
      await onSuccess();
    },
    onError: (error: Error) => onError(error.message),
  });

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>新建封禁</DialogTitle>
          <DialogDescription>封禁会记录当前管理员账号作为签发者。目标 SteamID64 必须已在库中登记。</DialogDescription>
        </DialogHeader>

        <div className="space-y-3">
          <div className="space-y-1.5">
            <Label htmlFor="ban-steamid">SteamID64</Label>
            <Input
              id="ban-steamid"
              value={steamId64}
              onChange={(event) => setSteamId64(event.target.value)}
              placeholder="7656119..."
              className="font-mono"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="ban-type">类型</Label>
            <Input id="ban-type" value={banType} onChange={(event) => setBanType(event.target.value)} />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="ban-reason">原因</Label>
            <Input
              id="ban-reason"
              value={reason}
              onChange={(event) => setReason(event.target.value)}
              placeholder="作弊 / 违规 / 其它"
            />
          </div>
          <div className="space-y-1.5">
            <Label htmlFor="ban-expires">到期时间（留空 = 永久）</Label>
            <Input
              id="ban-expires"
              type="datetime-local"
              value={expiresAt}
              onChange={(event) => setExpiresAt(event.target.value)}
            />
          </div>
        </div>

        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" size="sm">
              取消
            </Button>
          </DialogClose>
          <Button
            size="sm"
            disabled={mutation.isPending || !steamId64.trim() || !reason.trim()}
            onClick={() => mutation.mutate()}
          >
            {mutation.isPending ? '提交中…' : '确认封禁'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

function RevokeBanDialog({
  ban,
  onClose,
  onError,
  onSuccess,
}: {
  ban: BanRow | null;
  onClose: () => void;
  onError: (message: string) => void;
  onSuccess: () => void | Promise<void>;
}) {
  const [reason, setReason] = useState('');

  const mutation = useMutation({
    mutationFn: () => api.bans.revoke(ban!.id, { reason: reason.trim() }),
    onSuccess: async () => {
      setReason('');
      onClose();
      await onSuccess();
    },
    onError: (error: Error) => onError(error.message),
  });

  return (
    <Dialog
      open={ban !== null}
      onOpenChange={(open) => {
        if (!open) onClose();
      }}
    >
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>解除封禁</DialogTitle>
          <DialogDescription>
            {ban ? `${ban.display_name}（${ban.steam_id64}）的封禁将被标记为已撤销，原始记录保留。` : ''}
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-1.5">
          <Label htmlFor="revoke-reason">解封原因</Label>
          <Input
            id="revoke-reason"
            value={reason}
            onChange={(event) => setReason(event.target.value)}
            placeholder="误封 / 申诉通过 / 到期"
          />
        </div>
        <DialogFooter>
          <DialogClose asChild>
            <Button variant="outline" size="sm">
              取消
            </Button>
          </DialogClose>
          <Button size="sm" disabled={mutation.isPending || !reason.trim()} onClick={() => mutation.mutate()}>
            {mutation.isPending ? '提交中…' : '确认解封'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
