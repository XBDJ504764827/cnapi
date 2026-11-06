import { RefreshCw } from 'lucide-react';

import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';

export function ErrorState({
  title,
  description,
  onRetry,
}: {
  title: string;
  description?: string;
  onRetry?: () => void;
}) {
  return (
    <Callout
      variant="danger"
      title={title}
      description={description}
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
