import { BookOpen, ExternalLink, FileJson } from 'lucide-react';

import { PageHeader } from '@/components/common/page-header';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table';
import { apiDocsUrl, openApiDocumentUrl } from '@/lib/env';

const endpoints = [
  { method: 'GET', path: '/health', description: 'API 健康检查（进程可响应）', tag: 'health' },
  { method: 'GET', path: '/health/live', description: '进程存活检查', tag: 'health' },
  { method: 'GET', path: '/health/ready', description: '依赖就绪检查（PostgreSQL / Redis）', tag: 'health' },
  { method: 'GET', path: '/api/v1/system/info', description: '系统版本与运行时长', tag: 'system' },
  { method: 'GET', path: '/api/v1/system/status', description: '基础服务实时连接状态', tag: 'system' },
  { method: 'GET', path: '/api/v1/system/events', description: '最近系统事件（limit 1-100）', tag: 'system' },
] as const;

/** Entry point to the backend generated API documentation. */
export function ApiDocsPage() {
  return (
    <div className="space-y-6">
      <PageHeader
        title="API 文档"
        description="接口文档由后端 Utoipa 根据路由与数据结构自动生成，始终与实现保持一致。"
        meta={<Badge variant="outline">OpenAPI 3.1</Badge>}
        actions={
          <>
            <Button asChild>
              <a href={apiDocsUrl} target="_blank" rel="noreferrer">
                <BookOpen />
                打开 Swagger UI
                <ExternalLink className="size-3 opacity-70" />
              </a>
            </Button>
            <Button variant="outline" asChild>
              <a href={openApiDocumentUrl} target="_blank" rel="noreferrer">
                <FileJson />
                OpenAPI JSON
              </a>
            </Button>
          </>
        }
      />

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">已实现的接口</CardTitle>
          <CardDescription>阶段 1 仅包含健康检查与系统信息接口，业务接口将在后续阶段补充。</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-20">方法</TableHead>
                <TableHead>路径</TableHead>
                <TableHead className="hidden sm:table-cell">说明</TableHead>
                <TableHead className="hidden md:table-cell">分组</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {endpoints.map((endpoint) => (
                <TableRow key={endpoint.path}>
                  <TableCell>
                    <Badge variant="secondary" className="font-mono">
                      {endpoint.method}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    <a
                      href={endpoint.path}
                      target="_blank"
                      rel="noreferrer"
                      className="font-mono text-xs underline-offset-4 hover:underline"
                    >
                      {endpoint.path}
                    </a>
                  </TableCell>
                  <TableCell className="hidden text-sm text-muted-foreground sm:table-cell">
                    {endpoint.description}
                  </TableCell>
                  <TableCell className="hidden md:table-cell">
                    <Badge variant="outline">{endpoint.tag}</Badge>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>

      <p className="text-xs text-muted-foreground">
        统一响应结构：成功为 <code className="font-mono">{'{ success, data, message }'}</code>，失败为{' '}
        <code className="font-mono">{'{ success, error, message }'}</code>，并使用正确的 HTTP 状态码。
      </p>
    </div>
  );
}
