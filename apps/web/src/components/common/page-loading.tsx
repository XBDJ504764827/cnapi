import { Skeleton } from '@/components/ui/skeleton';

/** Neutral loading placeholder used by lazy routes and query skeletons. */
export function PageLoading({ label = '加载中' }: { label?: string }) {
  return (
    <div className="space-y-4" role="status" aria-label={label}>
      <div className="space-y-2">
        <Skeleton className="h-7 w-40" />
        <Skeleton className="h-4 w-72" />
      </div>
      <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-4">
        {Array.from({ length: 4 }).map((_, index) => (
          <Skeleton key={index} className="h-24 rounded-xl" />
        ))}
      </div>
      <div className="grid gap-4 lg:grid-cols-2">
        <Skeleton className="h-56 rounded-xl" />
        <Skeleton className="h-56 rounded-xl" />
      </div>
      <span className="sr-only">{label}</span>
    </div>
  );
}
