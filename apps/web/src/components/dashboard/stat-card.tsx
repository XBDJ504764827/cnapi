import type { LucideIcon } from 'lucide-react';

import { Card, CardContent } from '@/components/ui/card';

interface StatCardProps {
  label: string;
  value: string;
  icon: LucideIcon;
  hint?: string;
}

/**
 * Compact KPI card. Values are always passed in from real API data or `—`
 * when the corresponding endpoint does not exist yet.
 */
export function StatCard({ label, value, icon: Icon, hint }: StatCardProps) {
  return (
    <Card className="gap-0 py-4">
      <CardContent className="flex items-start justify-between gap-3 px-4">
        <div className="min-w-0 space-y-1">
          <p className="text-xs text-muted-foreground">{label}</p>
          <p className="truncate text-2xl font-semibold tabular-nums">{value}</p>
          {hint ? <p className="text-[11px] leading-snug text-muted-foreground/80">{hint}</p> : null}
        </div>
        <span className="flex size-9 shrink-0 items-center justify-center rounded-md bg-muted text-muted-foreground">
          <Icon className="size-4" aria-hidden="true" />
        </span>
      </CardContent>
    </Card>
  );
}
