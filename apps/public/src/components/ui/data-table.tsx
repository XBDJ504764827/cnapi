import type { ReactNode } from 'react';

import { ErrorState } from '@/components/common/error-state';
import { EmptyState } from '@/components/common/empty-state';
import { Skeleton } from '@/components/ui/skeleton';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { cn } from '@/lib/utils';

export interface DataTableColumn<T> {
  id: string;
  header: ReactNode;
  cell: (row: T) => ReactNode;
  className?: string;
  headerClassName?: string;
}

interface DataTableProps<T> {
  columns: DataTableColumn<T>[];
  rows: T[];
  getRowKey: (row: T) => string | number;
  isLoading?: boolean;
  error?: { title: string; description?: string; onRetry?: () => void } | null;
  empty?: ReactNode;
  /** Minimum table width so horizontal scrolling kicks in on small screens. */
  minWidthClass?: string;
  className?: string;
}

/**
 * Shared table shell: header, skeleton rows, error row and empty row.
 * Pages stay responsible for toolbars, pagination and real data.
 */
export function DataTable<T>({
  columns,
  rows,
  getRowKey,
  isLoading = false,
  error = null,
  empty,
  minWidthClass = 'min-w-[760px]',
  className,
}: DataTableProps<T>) {
  // Error and empty states are rendered outside the scroll container: a
  // horizontally scrollable table would clip a centred empty state on mobile.
  if (error) {
    return (
      <div className="p-4">
        <ErrorState title={error.title} description={error.description} onRetry={error.onRetry} />
      </div>
    );
  }

  if (!isLoading && rows.length === 0) {
    // Rendering the empty state outside the horizontal scroll container keeps
    // it readable on tablets and phones, where the table can be wider than the
    // viewport.
    return (
      <div className="p-4">
        {empty ?? <EmptyState compact title="暂无数据" description="当前筛选条件下没有可展示的数据。" />}
      </div>
    );
  }

  return (
    <Table className={cn(minWidthClass, className)} aria-busy={isLoading || undefined}>
      <TableHeader>
        <TableRow className="hover:bg-transparent">
          {columns.map((column) => (
            <TableHead key={column.id} className={column.headerClassName}>
              {column.header}
            </TableHead>
          ))}
        </TableRow>
      </TableHeader>
      <TableBody>
        {isLoading
          ? Array.from({ length: 5 }).map((_, rowIndex) => (
              <TableRow key={`skeleton-${rowIndex}`} className="hover:bg-transparent">
                {columns.map((column) => (
                  <TableCell key={column.id}>
                    <Skeleton className="h-4 w-full max-w-32" />
                  </TableCell>
                ))}
              </TableRow>
            ))
          : rows.map((row) => (
              <TableRow key={getRowKey(row)}>
                {columns.map((column) => (
                  <TableCell key={column.id} className={column.className}>
                    {column.cell(row)}
                  </TableCell>
                ))}
              </TableRow>
            ))}
      </TableBody>
    </Table>
  );
}
