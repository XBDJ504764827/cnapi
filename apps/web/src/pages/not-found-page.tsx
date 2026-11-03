import { Compass, Home } from 'lucide-react';
import { Link } from 'react-router';

import { Button } from '@/components/ui/button';
import { PageHeader } from '@/components/common/page-header';
import { EmptyState } from '@/components/common/empty-state';

export function NotFoundPage() {
  return (
    <div className="space-y-6">
      <PageHeader title="页面不存在" description="请求的地址没有对应的页面，可能已被移动或尚未实现。" />
      <EmptyState
        icon={Compass}
        title="404 · 未找到页面"
        description="请检查地址是否正确，或返回系统概览继续操作。"
        action={
          <Button asChild variant="outline">
            <Link to="/dashboard">
              <Home />
              返回系统概览
            </Link>
          </Button>
        }
      />
    </div>
  );
}
