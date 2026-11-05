import type * as React from 'react';

import { cn } from '@/public/lib/utils';

/** Small keycap used in shortcuts and command hints. */
export function Kbd({ className, ...props }: React.ComponentProps<'kbd'>) {
  return (
    <kbd
      className={cn(
        'inline-flex h-5 min-w-5 items-center justify-center rounded border border-border bg-surface-elevated px-1 font-mono text-[10px] leading-none text-muted-foreground',
        className,
      )}
      {...props}
    />
  );
}
