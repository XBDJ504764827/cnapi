import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function PlayersPage() {
  return (
    <ModulePlaceholder
      title="玩家管理"
      description="检索 KZ 玩家档案、等级、积分与历史成绩。当前阶段仅提供页面结构。"
      planned={[
        '按 SteamID / 昵称检索玩家并分页展示',
        '玩家详情：KZ 等级、积分、完成地图与最佳成绩',
        '玩家数据同步状态与异常标记',
      ]}
    />
  );
}
