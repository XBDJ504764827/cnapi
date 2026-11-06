import { Tabs as TabsPrimitive } from 'radix-ui';
import type * as React from 'react';

import { cn } from '@/public/lib/utils';

export function Tabs(props: React.ComponentProps<typeof TabsPrimitive.Root>) {
  return <TabsPrimitive.Root {...props} />;
}

export function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>) {
  return <TabsPrimitive.List className={cn('flex items-center gap-5 border-b border-border', className)} {...props} />;
}

export function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      className={cn(
        'relative -mb-px inline-flex h-9 cursor-pointer items-center gap-1.5 border-b-2 border-transparent px-0.5 text-[13px] font-medium text-muted-foreground transition-colors outline-none select-none',
        'hover:text-foreground focus-visible:text-foreground',
        'data-[state=active]:border-primary data-[state=active]:text-foreground',
        'disabled:pointer-events-none disabled:opacity-50 [&_svg]:size-3.5',
        className,
      )}
      {...props}
    />
  );
}

export function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return <TabsPrimitive.Content className={cn('mt-4 outline-none', className)} {...props} />;
}
