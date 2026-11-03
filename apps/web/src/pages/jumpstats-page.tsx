import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function JumpstatsPage() {
  return (
    <ModulePlaceholder
      title="Jumpstats"
      description="浏览与筛选玩家 Jumpstats 数据。当前阶段仅提供页面结构。"
      planned={['Jumpstats 列表：按玩家、地图、跳跃类型筛选', '数值分布与异常值检测', '与 Replay 的关联查看']}
    />
  );
}
