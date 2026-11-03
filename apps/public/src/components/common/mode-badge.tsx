import { Badge } from '@/components/ui/badge';
import { modeLabel } from '@/lib/format';

export function ModeBadge({ code }: { code: string }) {
  return (
    <Badge variant="outline" className="font-normal">
      {modeLabel(code)}
    </Badge>
  );
}
