import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { PublicModePayload } from '@cngokz/shared-types';

export function ModeTabs({
  modes,
  value,
  onChange,
  isLoading,
  includeAll = false,
}: {
  modes: PublicModePayload[] | undefined;
  value: string;
  onChange: (value: string) => void;
  isLoading?: boolean;
  includeAll?: boolean;
}) {
  if (isLoading) return <Skeleton className="mb-3 h-8 w-64" />;

  return (
    <Tabs value={value} onValueChange={onChange}>
      <TabsList className="mb-3">
        {includeAll ? <TabsTrigger value="all">全部模式</TabsTrigger> : null}
        {(modes ?? []).map((mode) => (
          <TabsTrigger key={mode.code} value={mode.code}>
            {mode.display_name}
          </TabsTrigger>
        ))}
      </TabsList>
    </Tabs>
  );
}
