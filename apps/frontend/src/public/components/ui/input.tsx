import type * as React from 'react';

import { cn } from '@/public/lib/utils';

export function Input({ className, type, ...props }: React.ComponentProps<'input'>) {
  return (
    <input
      type={type}
      data-slot="input"
      className={cn(
        'h-9 w-full min-w-0 rounded-md border border-border bg-surface-elevated/60 px-3 py-1 text-[13px] text-foreground shadow-xs transition-[color,box-shadow,border-color] outline-none',
        'placeholder:text-muted-foreground/70',
        'focus-visible:border-ring/70 focus-visible:ring-2 focus-visible:ring-ring/20',
        'disabled:cursor-not-allowed disabled:opacity-50',
        'aria-invalid:border-danger aria-invalid:ring-danger/20',
        className,
      )}
      {...props}
    />
  );
}
