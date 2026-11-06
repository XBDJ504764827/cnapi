import { useQuery } from '@tanstack/react-query';
import { useState } from 'react';

import { Pagination } from '@/public/components/ui/pagination';
import { Card } from '@/public/components/ui/card';
import { RecordDetailDialog } from '@/public/components/records/record-detail-dialog';
import { RecordTable } from '@/public/components/records/record-table';
import { useListParams } from '@/public/hooks/use-list-params';
import { publicQueries } from '@/public/services/public-service';
import type { RecordSummaryPayload } from '@cngokz/shared-types';

export function PlayerRecordHistory({ steamid, mode, timeType }: { steamid: string; mode: string; timeType: string }) {
  const list = useListParams({ pageKey: 'r_page', pageSizeKey: 'r_page_size' });
  const [selected, setSelected] = useState<RecordSummaryPayload | null>(null);

  const query = useQuery(
    publicQueries.playerRecords(steamid, {
      mode,
      time_type: timeType,
      page: list.page,
      page_size: list.pageSize,
    }),
  );

  return (
    <Card className="overflow-hidden">
      <div className="flex flex-wrap items-center justify-between gap-2 border-b border-border/70 px-4 py-3">
        <p className="text-[13px] font-medium">全部成绩</p>
        <p className="text-[11px] text-muted-foreground">
          当前：{timeType === 'all' ? '全部成绩类型' : timeType.toUpperCase()}
        </p>
      </div>
      <RecordTable
        records={query.data?.items}
        isLoading={query.isLoading}
        errorMessage={query.isError ? query.error.message : undefined}
        onRetry={() => void query.refetch()}
        onSelect={setSelected}
        showMap
        showRank
        emptyTitle="暂无成绩"
        emptyDescription="该玩家在当前模式下还没有成绩。"
      />
      <div className="border-t border-border/70 px-4 py-3">
        <Pagination
          page={query.data?.meta.page ?? list.page}
          pageSize={query.data?.meta.page_size ?? list.pageSize}
          total={query.data?.meta.total ?? 0}
          onPageChange={list.setPage}
          onPageSizeChange={list.setPageSize}
        />
      </div>
      <RecordDetailDialog
        record={selected}
        onOpenChange={(open) => {
          if (!open) setSelected(null);
        }}
      />
    </Card>
  );
}
