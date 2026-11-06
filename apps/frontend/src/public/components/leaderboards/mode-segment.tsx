import { Skeleton } from '@/public/components/ui/skeleton';
import { cn } from '@/public/lib/utils';
import type { PublicModePayload } from '@cngokz/shared-types';

/**
 * Mode switcher for the leaderboard page. A compact segmented control fits the
 * dense "ranking console" header better than a full-width underline tab bar.
 */
export function ModeSegment({
  modes,
  value,
  onChange,
  isLoading,
}: {
  modes: PublicModePayload[] | undefined;
  value: string;
  onChange: (value: string) => void;
  isLoading?: boolean;
}) {
  if (isLoading) return <Skeleton className="h-9 w-72 rounded-md" />;

  return (
    <div
      role="radiogroup"
      aria-label="游戏模式"
      className="inline-flex max-w-full items-center gap-0.5 overflow-x-auto rounded-md border border-border bg-surface-elevated/40 p-0.5"
    >
      {(modes ?? []).map((mode) => {
        const selected = mode.code === value;
        return (
          <button
            key={mode.code}
            type="button"
            role="radio"
            aria-checked={selected}
            onClick={() => onChange(mode.code)}
            className={cn(
              'h-8 shrink-0 cursor-pointer rounded-sm px-3 text-xs font-medium transition-colors outline-none select-none',
              selected
                ? 'bg-primary/15 text-primary shadow-xs'
                : 'text-muted-foreground hover:bg-surface-hover hover:text-foreground',
            )}
          >
            {mode.display_name}
          </button>
        );
      })}
    </div>
  );
}
