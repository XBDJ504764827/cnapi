import { RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { Callout } from '@/components/ui/callout';
import { cn } from '@/lib/utils';

interface ErrorStateProps {
  title: string;
  description?: string;
  onRetry?: () => void;
  className?: string;
}

export function ErrorState({ title, description, onRetry, className }: ErrorStateProps) {
  return (
    <Callout
      variant="danger"
      title={title}
      description={description}
      className={cn('items-center', className)}
      action={
        onRetry ? (
          <Button variant="outline" size="sm" onClick={onRetry}>
            <RefreshCw />
            重试
          </Button>
        ) : undefined
      }
    />
  );
}
