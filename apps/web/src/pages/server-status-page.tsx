import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function ServerStatusPage() {
  return (
    <ModulePlaceholder
      title="服务器状态"
      description="查看游戏服务器在线状态、玩家数量与最近心跳。当前阶段仅提供页面结构。"
      planned={['服务器心跳与在线时长', '在线玩家数与当前地图', '失联服务器告警']}
    />
  );
}
