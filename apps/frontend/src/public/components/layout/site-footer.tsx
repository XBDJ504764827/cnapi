import { ExternalLink } from 'lucide-react';
import { Link } from 'react-router';

import { apiDocsUrl } from '@/public/lib/env';

export function SiteFooter() {
  return (
    <footer className="mt-auto border-t border-border/70">
      <div className="mx-auto flex w-full max-w-7xl flex-col gap-3 px-4 py-6 text-xs text-muted-foreground sm:px-6 md:flex-row md:items-center md:justify-between lg:px-8">
        <div className="space-y-1">
          <p className="font-medium text-foreground">CNGOKZ · CS:GO / CS2 KZ 数据平台</p>
          <p>数据来自 CNGOKZAPI 公开数据库；未评分/未实现字段显示为 “—”，不展示模拟数据。</p>
        </div>
        <nav className="flex flex-wrap items-center gap-x-4 gap-y-1" aria-label="页脚导航">
          <Link to="/leaderboards" className="hover:text-foreground">
            排行榜
          </Link>
          <Link to="/points" className="hover:text-foreground">
            积分榜
          </Link>
          <Link to="/players" className="hover:text-foreground">
            玩家
          </Link>
          <Link to="/maps" className="hover:text-foreground">
            地图
          </Link>
          <Link to="/records" className="hover:text-foreground">
            成绩
          </Link>
          <a
            href={apiDocsUrl}
            target="_blank"
            rel="noreferrer"
            className="inline-flex items-center gap-1 hover:text-foreground"
          >
            API 文档
            <ExternalLink className="size-3" />
          </a>
          <a href="/admin/" className="hover:text-foreground">
            管理后台
          </a>
        </nav>
      </div>
    </footer>
  );
}
