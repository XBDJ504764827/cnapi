import type { LucideIcon } from 'lucide-react';

import { Card } from '@/components/ui/card';

interface StatCardProps {
  label: string;
  value: string;
  icon: LucideIcon;
  hint?: string;
  /** Marks a metric that has no backing endpoint yet. */
  unavailable?: boolean;
}

/**
 * Compact KPI card. Values are always passed in from real API data or `—`
 * when the corresponding endpoint does not exist yet.
 */
export function StatCard({ label, value, icon: Icon, hint, unavailable = false }: StatCardProps) {
  return (
    <Card className="gap-0 p-4">
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0">
          <p className="text-xs text-muted-foreground">{label}</p>
          <p className="mt-2 truncate text-[26px] leading-none font-semibold tracking-tight tabular-nums">{value}</p>
        </div>
        <span className="flex size-8 shrink-0 items-center justify-center rounded-md border border-border bg-surface-elevated text-muted-foreground">
          <Icon className="size-4" aria-hidden="true" />
        </span>
      </div>
      <div className="mt-3 flex items-center gap-1.5">
        {unavailable ? (
          <span className="inline-flex items-center gap-1 rounded border border-border px-1.5 py-0.5 text-[10px] text-muted-foreground">
            接口未接入
          </span>
        ) : null}
        {hint ? <p className="truncate text-[11px] text-muted-foreground">{hint}</p> : null}
      </div>
    </Card>
  );
}
