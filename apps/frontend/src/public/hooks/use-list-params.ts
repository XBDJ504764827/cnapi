import { useSearchParams } from 'react-router';

/**
 * URL-backed list state. Filters reset the page; page size is persisted in the
 * URL so views are shareable and the back button works.
 */
export function useListParams(options: { pageKey?: string; pageSizeKey?: string } = {}) {
  const { pageKey = 'page', pageSizeKey = 'page_size' } = options;
  const [params, setParams] = useSearchParams();
  const page = Math.max(1, Number(params.get(pageKey) ?? '1') || 1);
  const pageSize = Math.min(100, Math.max(1, Number(params.get(pageSizeKey) ?? '20') || 20));

  const read = (key: string, fallback = '') => params.get(key) ?? fallback;

  const setFilter = (key: string, value: string, fallback = '') => {
    const next = new URLSearchParams(params);
    if (!value || value === fallback) next.delete(key);
    else next.set(key, value);
    next.delete(pageKey);
    setParams(next, { replace: true });
  };

  const setFilters = (values: Record<string, string | undefined>) => {
    const next = new URLSearchParams(params);
    for (const [key, value] of Object.entries(values)) {
      if (value) next.set(key, value);
      else next.delete(key);
    }
    next.delete(pageKey);
    setParams(next, { replace: true });
  };

  const setPage = (nextPage: number) => {
    const next = new URLSearchParams(params);
    if (nextPage <= 1) next.delete(pageKey);
    else next.set(pageKey, String(nextPage));
    setParams(next, { replace: true });
  };

  const setPageSize = (value: number) => {
    const next = new URLSearchParams(params);
    if (value === 20) next.delete(pageSizeKey);
    else next.set(pageSizeKey, String(value));
    next.delete(pageKey);
    setParams(next, { replace: true });
  };

  const reset = () => setParams(new URLSearchParams(), { replace: true });

  return { read, setFilter, setFilters, setPage, setPageSize, reset, page, pageSize, params };
}
