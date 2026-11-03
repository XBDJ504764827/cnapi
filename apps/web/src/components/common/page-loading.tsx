import { Skeleton } from '@/components/ui/skeleton';

/** Neutral loading placeholder used by lazy routes. */
export function PageLoading({ label = '加载中' }: { label?: string }) {
  return (
    <div className="space-y-6" role="status" aria-label={label}>
      <div className="space-y-2">
        <Skeleton className="h-6 w-40" />
        <Skeleton className="h-4 w-80 max-w-full" />
      </div>
      <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-24 rounded-lg" />
        ))}
      </div>
      <div className="grid gap-3 lg:grid-cols-3">
        <Skeleton className="h-64 rounded-lg lg:col-span-2" />
        <Skeleton className="h-64 rounded-lg" />
      </div>
      <span className="sr-only">{label}</span>
    </div>
  );
}
