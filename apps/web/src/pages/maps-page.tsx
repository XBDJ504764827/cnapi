import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function MapsPage() {
  return (
    <ModulePlaceholder
      title="地图管理"
      description="管理 KZ 地图元数据、难度分级与记录数量统计。当前阶段仅提供页面结构。"
      planned={[
        '地图列表与筛选（难度、类型、记录数）',
        '地图详情：作者、关卡数、全局最佳成绩',
        '地图元数据维护与重复地图合并',
      ]}
    />
  );
}
