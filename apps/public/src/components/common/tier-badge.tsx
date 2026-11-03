import { cn } from '@/lib/utils';

const tierClasses: Record<number, string> = {
  1: 'border-info/25 bg-info/10 text-info',
  2: 'border-info/30 bg-info/15 text-info',
  3: 'border-primary/25 bg-primary/10 text-primary',
  4: 'border-primary/35 bg-primary/15 text-primary',
  5: 'border-warning/30 bg-warning/10 text-warning',
  6: 'border-warning/40 bg-warning/15 text-warning',
  7: 'border-danger/30 bg-danger/10 text-danger',
  8: 'border-danger/40 bg-danger/15 text-danger',
  9: 'border-danger/50 bg-danger/20 text-danger',
  10: 'border-danger/60 bg-danger/25 text-danger',
};

export function TierBadge({ tier, className }: { tier: number | null | undefined; className?: string }) {
  return (
    <span
      className={cn(
        'inline-flex h-5 min-w-8 items-center justify-center rounded border px-1.5 font-mono text-[11px] font-medium tabular-nums',
        tier
          ? (tierClasses[tier] ?? 'border-border bg-surface-elevated text-muted-foreground')
          : 'border-border text-muted-foreground',
        className,
      )}
      title={tier ? `Tier ${tier}` : '未评级'}
    >
      {tier ? `T${tier}` : '—'}
    </span>
  );
}
