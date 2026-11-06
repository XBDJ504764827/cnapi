import { cn } from '@/admin/lib/utils';
import type { StatusTone } from '@/admin/types';

const toneStyles: Record<StatusTone, { dot: string; text: string; label: string }> = {
  up: { dot: 'bg-success', text: 'text-success', label: '正常' },
  down: { dot: 'bg-danger', text: 'text-danger', label: '不可用' },
  unknown: { dot: 'bg-muted-foreground/60', text: 'text-muted-foreground', label: '未知' },
};

interface StatusIndicatorProps {
  tone: StatusTone;
  label?: string;
  /** Adds a soft pulse (used for pending / in-progress states). */
  pulse?: boolean;
  showLabel?: boolean;
  className?: string;
}

/** Dot + optional label, used for services, servers and record states. */
export function StatusIndicator({ tone, label, pulse = false, showLabel = true, className }: StatusIndicatorProps) {
  const styles = toneStyles[tone];

  return (
    <span className={cn('inline-flex items-center gap-1.5 text-[13px] font-medium whitespace-nowrap', className)}>
      <span className={cn('relative flex size-1.5 shrink-0 rounded-full', styles.dot)} aria-hidden="true">
        {pulse ? (
          <span className={cn('absolute inline-flex size-full animate-ping rounded-full opacity-60', styles.dot)} />
        ) : null}
      </span>
      {showLabel ? <span className={cn(styles.text, 'text-[12px]')}>{label ?? styles.label}</span> : null}
    </span>
  );
}

export { toneStyles };
