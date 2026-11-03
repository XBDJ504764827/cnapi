import { BookOpen, ExternalLink, FileJson } from 'lucide-react';

import { Callout } from '@/components/ui/callout';
import { CodeBlock } from '@/components/common/code-block';
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

const successEnvelope = `{
  "success": true,
  "data": { },
  "message": "ok"
}`;

const errorEnvelope = `{
  "success": false,
  "error": {
    "code": "validation_error",
    "details": null
  },
  "message": "limit must be between 1 and 100"
}`;

/** Entry point to the backend generated API documentation. */
export function ApiDocsPage() {
  return (
    <div className="space-y-5">
      <PageHeader
        title="API 文档"
        description="接口文档由后端 Utoipa 根据路由与数据结构自动生成，始终与实现保持一致。"
        meta={<Badge variant="outline">OpenAPI 3.1</Badge>}
        actions={
          <>
            <Button size="sm" asChild>
              <a href={apiDocsUrl} target="_blank" rel="noreferrer">
                <BookOpen />
                打开 Swagger UI
                <ExternalLink className="size-3 opacity-70" />
              </a>
            </Button>
            <Button variant="outline" size="sm" asChild>
              <a href={openApiDocumentUrl} target="_blank" rel="noreferrer">
                <FileJson />
                OpenAPI JSON
              </a>
            </Button>
          </>
        }
      />

      <Callout
        variant="info"
        title="文档自动生成"
        description="修改后端 handler 或 DTO 后运行 scripts/gen-api.sh，即可同步 OpenAPI 文档与前端 TypeScript 类型。"
      />

      <Card>
        <CardHeader>
          <CardTitle>已实现的接口</CardTitle>
          <CardDescription>当前仅包含健康检查与系统信息接口，业务接口将在后续阶段补充。</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="overflow-x-auto rounded-md border border-border/70">
            <Table className="min-w-[720px]">
              <TableHeader>
                <TableRow className="hover:bg-transparent">
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
                      <Badge variant={endpoint.method === 'GET' ? 'info' : 'primary'} className="font-mono">
                        {endpoint.method}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      <a
                        href={endpoint.path}
                        target="_blank"
                        rel="noreferrer"
                        className="font-mono text-xs underline-offset-4 hover:text-primary hover:underline"
                      >
                        {endpoint.path}
                      </a>
                    </TableCell>
                    <TableCell className="hidden text-xs text-muted-foreground sm:table-cell">
                      {endpoint.description}
                    </TableCell>
                    <TableCell className="hidden md:table-cell">
                      <Badge variant="outline">{endpoint.tag}</Badge>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>

      <div className="grid gap-3 lg:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>成功响应</CardTitle>
            <CardDescription>所有 2xx 响应使用统一信封</CardDescription>
          </CardHeader>
          <CardContent>
            <CodeBlock code={successEnvelope} label="200 OK" />
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>失败响应</CardTitle>
            <CardDescription>HTTP 状态码与 error.code 同时表达语义</CardDescription>
          </CardHeader>
          <CardContent>
            <CodeBlock code={errorEnvelope} label="400 Bad Request" />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
