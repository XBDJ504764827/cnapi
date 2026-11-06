import type { LucideIcon } from 'lucide-react';

import { Card } from '@/public/components/ui/card';
import { Skeleton } from '@/public/components/ui/skeleton';
import { formatNumber } from '@/public/lib/format';

export function StatCard({
  label,
  value,
  icon: Icon,
  hint,
  isLoading,
}: {
  label: string;
  value: number | null | undefined;
  icon: LucideIcon;
  hint?: string;
  isLoading?: boolean;
}) {
  return (
    <Card className="gap-0 p-4">
      <div className="flex items-start justify-between gap-2">
        <p className="text-xs text-muted-foreground">{label}</p>
        <Icon className="size-4 text-muted-foreground/60" aria-hidden="true" />
      </div>
      {isLoading ? (
        <Skeleton className="mt-2 h-8 w-20" />
      ) : (
        <p className="mt-2 text-[28px] leading-none font-semibold tracking-tight tabular-nums">{formatNumber(value)}</p>
      )}
      {hint ? <p className="mt-2 text-[11px] text-muted-foreground">{hint}</p> : null}
    </Card>
  );
}
