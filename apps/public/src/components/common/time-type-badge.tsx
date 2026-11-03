import type { TimeType } from '@cngokz/shared-types';

import { Badge } from '@/components/ui/badge';

export function TimeTypeBadge({ value, className }: { value: TimeType | null | undefined; className?: string }) {
  if (!value) return <span className="text-xs text-muted-foreground">—</span>;
  return (
    <Badge variant={value === 'pro' ? 'success' : 'warning'} className={className}>
      {value === 'pro' ? 'PRO' : 'NUB'}
    </Badge>
  );
}
