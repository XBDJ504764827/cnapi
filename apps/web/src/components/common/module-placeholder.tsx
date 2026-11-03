import { PackageOpen } from 'lucide-react';

import { EmptyState } from '@/components/common/empty-state';
import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

interface ModulePlaceholderProps {
  title: string;
  description: string;
  /** Planned capabilities, shown as information only (no fake actions). */
  planned: string[];
}

/**
 * Layout used by modules that are not implemented yet. It deliberately contains
 * no fake data and no buttons that would pretend to perform an operation.
 */
export function ModulePlaceholder({ title, description, planned }: ModulePlaceholderProps) {
  return (
    <div className="space-y-6">
      <PageHeader title={title} description={description} meta={<Badge variant="outline">阶段 2+</Badge>} />

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">规划中的能力</CardTitle>
          <CardDescription>以下内容尚未实现，当前阶段只提供页面结构与入口。</CardDescription>
        </CardHeader>
        <CardContent>
          <ul className="list-inside list-disc space-y-1.5 text-sm text-muted-foreground">
            {planned.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </CardContent>
      </Card>

      <EmptyState
        icon={PackageOpen}
        title="暂无数据"
        description="该模块的数据接口尚未开放，因此这里没有可展示的内容。"
        hint="接口接入后，此页面会展示真实数据，不会使用模拟数据填充。"
      />
    </div>
  );
}
