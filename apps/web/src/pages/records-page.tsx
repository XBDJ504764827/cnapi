import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function RecordsPage() {
  return (
    <ModulePlaceholder
      title="成绩管理"
      description="查看与审核玩家成绩记录，处理异常成绩与申诉。当前阶段仅提供页面结构。"
      planned={['成绩列表：按地图、玩家、时间筛选', '成绩详情与 Replay 关联', '成绩审核状态与异常标记']}
    />
  );
}
