import { Badge } from '@/public/components/ui/badge';
import { modeLabel } from '@/public/lib/format';

export function ModeBadge({ code }: { code: string }) {
  return (
    <Badge variant="outline" className="font-normal">
      {modeLabel(code)}
    </Badge>
  );
}
