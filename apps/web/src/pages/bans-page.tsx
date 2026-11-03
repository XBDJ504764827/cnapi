import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function BansPage() {
  return (
    <ModulePlaceholder
      title="封禁记录"
      description="查看封禁记录、封禁来源与申诉状态。当前阶段仅提供页面结构。"
      planned={['封禁记录列表（SteamID、原因、时长）', '封禁与服务器 / 插件的同步状态', '申诉与解封流程']}
    />
  );
}
