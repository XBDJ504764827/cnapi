import { cva, type VariantProps } from 'class-variance-authority';
import type * as React from 'react';

import { cn } from '@/admin/lib/utils';

const badgeVariants = cva(
  'inline-flex w-fit shrink-0 items-center justify-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px] leading-4 font-medium whitespace-nowrap [&_svg]:size-3',
  {
    variants: {
      variant: {
        default: 'border-border bg-secondary text-secondary-foreground',
        secondary: 'border-border bg-secondary text-secondary-foreground',
        primary: 'border-primary/25 bg-primary/12 text-primary',
        outline: 'border-border bg-transparent text-muted-foreground',
        success: 'border-success/25 bg-success/12 text-success',
        warning: 'border-warning/25 bg-warning/12 text-warning',
        danger: 'border-danger/25 bg-danger/12 text-danger',
        destructive: 'border-danger/25 bg-danger/12 text-danger',
        info: 'border-info/25 bg-info/12 text-info',
      },
    },
    defaultVariants: {
      variant: 'default',
    },
  },
);

export interface BadgeProps extends React.ComponentProps<'span'>, VariantProps<typeof badgeVariants> {}

export function Badge({ className, variant, ...props }: BadgeProps) {
  return <span data-slot="badge" className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { badgeVariants };
