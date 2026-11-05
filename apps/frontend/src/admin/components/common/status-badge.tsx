import { Badge } from '@/admin/components/ui/badge';
import { cn } from '@/admin/lib/utils';
import type { StatusTone } from '@/admin/types';

const toneConfig: Record<StatusTone, { label: string; variant: 'success' | 'danger' | 'outline'; dot: string }> = {
  up: { label: '正常', variant: 'success', dot: 'bg-success' },
  down: { label: '不可用', variant: 'danger', dot: 'bg-danger' },
  unknown: { label: '未知', variant: 'outline', dot: 'bg-muted-foreground' },
};

export function StatusBadge({ tone, label, className }: { tone: StatusTone; label?: string; className?: string }) {
  const config = toneConfig[tone];

  return (
    <Badge variant={config.variant} className={cn('gap-1.5', className)}>
      <span className={cn('size-1.5 rounded-full', config.dot)} aria-hidden="true" />
      {label ?? config.label}
    </Badge>
  );
}
