import { useState } from 'react';

import type { SortDirection } from '@/admin/types';

interface UseTableStateOptions {
  initialPageSize?: number;
  initialSort?: { id: string; direction: SortDirection } | null;
}

/**
 * Shared list state: page, page size, search term and sort. Search / page size
 * changes reset the page so results never point to an out-of-range page.
 */
export function useTableState({ initialPageSize = 20, initialSort = null }: UseTableStateOptions = {}) {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(initialPageSize);
  const [search, setSearchTerm] = useState('');
  const [sort, setSort] = useState<{ id: string; direction: SortDirection } | null>(initialSort);

  const setSearch = (value: string) => {
    setSearchTerm(value);
    setPage(1);
  };

  const changePageSize = (size: number) => {
    setPageSize(size);
    setPage(1);
  };

  const toggleSort = (id: string, defaultDirection: SortDirection = 'asc') => {
    setSort((previous) =>
      previous?.id === id
        ? { id, direction: previous.direction === 'asc' ? 'desc' : 'asc' }
        : { id, direction: defaultDirection },
    );
  };

  const resetFilters = () => {
    setSearchTerm('');
    setPage(1);
    setSort(initialSort);
  };

  return {
    page,
    pageSize,
    search,
    sort,
    setPage,
    setSearch,
    setPageSize: changePageSize,
    toggleSort,
    resetFilters,
  };
}
