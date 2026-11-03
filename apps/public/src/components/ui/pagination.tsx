import { ChevronLeft, ChevronRight } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { cn } from '@/lib/utils';

interface PaginationProps {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange?: (pageSize: number) => void;
  pageSizeOptions?: number[];
  className?: string;
}

/** Controlled pagination with page size selection and ellipsis windows. */
export function Pagination({
  page,
  pageSize,
  total,
  onPageChange,
  onPageSizeChange,
  pageSizeOptions = [10, 20, 50, 100],
  className,
}: PaginationProps) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const current = Math.min(Math.max(1, page), totalPages);
  const pages = pageItems(current, totalPages);

  return (
    <div className={cn('flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between', className)}>
      <p className="text-xs text-muted-foreground tabular-nums">
        共 {total.toLocaleString('zh-CN')} 条<span className="mx-2 text-border-strong">·</span>第 {current} /{' '}
        {totalPages} 页
      </p>

      <div className="flex flex-wrap items-center gap-2">
        {onPageSizeChange ? (
          <div className="flex items-center gap-1.5">
            <span className="text-xs text-muted-foreground">每页</span>
            <Select value={String(pageSize)} onValueChange={(value) => onPageSizeChange(Number(value))}>
              <SelectTrigger size="sm" className="w-16" aria-label="每页条数">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {pageSizeOptions.map((size) => (
                  <SelectItem key={size} value={String(size)}>
                    {size}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        <div className="flex items-center gap-0.5">
          <Button
            variant="ghost"
            size="icon-xs"
            onClick={() => onPageChange(Math.max(1, current - 1))}
            disabled={current <= 1 || total === 0}
            aria-label="上一页"
          >
            <ChevronLeft />
          </Button>

          <div className="hidden items-center gap-0.5 sm:flex">
            {pages.map((item, index) =>
              item === 'ellipsis' ? (
                <span key={`ellipsis-${index}`} className="px-1 text-xs text-muted-foreground">
                  …
                </span>
              ) : (
                <Button
                  key={item}
                  variant={item === current ? 'secondary' : 'ghost'}
                  size="icon-xs"
                  className="text-xs tabular-nums"
                  onClick={() => onPageChange(item)}
                  aria-label={`第 ${item} 页`}
                  aria-current={item === current ? 'page' : undefined}
                >
                  {item}
                </Button>
              ),
            )}
          </div>

          <Button
            variant="ghost"
            size="icon-xs"
            onClick={() => onPageChange(Math.min(totalPages, current + 1))}
            disabled={current >= totalPages || total === 0}
            aria-label="下一页"
          >
            <ChevronRight />
          </Button>
        </div>
      </div>
    </div>
  );
}

function pageItems(current: number, totalPages: number): Array<number | 'ellipsis'> {
  if (totalPages <= 7) {
    return Array.from({ length: totalPages }, (_, index) => index + 1);
  }

  const items: Array<number | 'ellipsis'> = [1];
  const start = Math.max(2, current - 1);
  const end = Math.min(totalPages - 1, current + 1);

  if (start > 2) items.push('ellipsis');
  for (let page = start; page <= end; page += 1) items.push(page);
  if (end < totalPages - 1) items.push('ellipsis');
  items.push(totalPages);

  return items;
}
