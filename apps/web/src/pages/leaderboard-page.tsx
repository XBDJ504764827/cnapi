import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function LeaderboardPage() {
  return (
    <ModulePlaceholder
      title="排行榜"
      description="全球排行榜与赛季排名视图。当前阶段仅提供页面结构。"
      planned={['按地图 / 全局划分的排行榜', '玩家排名变化与赛季归档', '排行榜缓存与刷新策略']}
    />
  );
}
