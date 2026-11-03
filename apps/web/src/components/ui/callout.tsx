import { CircleAlert, CircleCheck, Info, TriangleAlert } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import type * as React from 'react';

import { cn } from '@/lib/utils';

type CalloutVariant = 'info' | 'success' | 'warning' | 'danger' | 'neutral';

const variantStyles: Record<CalloutVariant, { wrapper: string; icon: string; Icon: LucideIcon }> = {
  info: { wrapper: 'border-info/25 bg-info/8', icon: 'text-info', Icon: Info },
  success: { wrapper: 'border-success/25 bg-success/8', icon: 'text-success', Icon: CircleCheck },
  warning: { wrapper: 'border-warning/25 bg-warning/8', icon: 'text-warning', Icon: TriangleAlert },
  danger: { wrapper: 'border-danger/25 bg-danger/8', icon: 'text-danger', Icon: CircleAlert },
  neutral: { wrapper: 'border-border bg-surface-elevated/60', icon: 'text-muted-foreground', Icon: Info },
};

interface CalloutProps {
  variant?: CalloutVariant;
  title: string;
  description?: React.ReactNode;
  action?: React.ReactNode;
  icon?: LucideIcon;
  className?: string;
}

/**
 * Inline notice used for "endpoint not implemented", degraded services and
 * other contextual messages. It never imitates a successful action.
 */
export function Callout({ variant = 'neutral', title, description, action, icon, className }: CalloutProps) {
  const styles = variantStyles[variant];
  const Icon = icon ?? styles.Icon;

  return (
    <div role="note" className={cn('flex gap-2.5 rounded-lg border px-3.5 py-3', styles.wrapper, className)}>
      <Icon className={cn('mt-0.5 size-4 shrink-0', styles.icon)} aria-hidden="true" />
      <div className="min-w-0 flex-1 space-y-0.5">
        <p className="text-[13px] font-medium text-foreground">{title}</p>
        {description ? <div className="text-xs leading-relaxed text-muted-foreground">{description}</div> : null}
      </div>
      {action ? <div className="ml-2 shrink-0 self-center">{action}</div> : null}
    </div>
  );
}
