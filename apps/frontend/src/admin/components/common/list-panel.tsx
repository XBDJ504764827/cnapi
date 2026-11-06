import type { ReactNode } from 'react';

import { Card } from '@/admin/components/ui/card';
import { cn } from '@/admin/lib/utils';

interface ListPanelProps {
  toolbar?: ReactNode;
  children: ReactNode;
  footer?: ReactNode;
  className?: string;
}

/** Card shell shared by all list pages: toolbar / table / pagination. */
export function ListPanel({ toolbar, children, footer, className }: ListPanelProps) {
  return (
    <Card className={cn('gap-0 overflow-hidden py-0', className)}>
      {toolbar ? <div className="border-b border-border/70 px-4 py-3">{toolbar}</div> : null}
      {children}
      {footer ? <div className="border-t border-border/70 px-4 py-3">{footer}</div> : null}
    </Card>
  );
}

interface FilterBarProps {
  children: ReactNode;
  actions?: ReactNode;
  className?: string;
}

/** Responsive filter row: controls on the left, secondary actions on the right. */
export function FilterBar({ children, actions, className }: FilterBarProps) {
  return (
    <div className={cn('flex flex-col gap-2 lg:flex-row lg:items-center lg:justify-between', className)}>
      <div className="flex flex-wrap items-center gap-2">{children}</div>
      {actions ? <div className="flex flex-wrap items-center gap-2">{actions}</div> : null}
    </div>
  );
}
