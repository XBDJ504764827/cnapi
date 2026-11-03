import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function ServersPage() {
  return (
    <ModulePlaceholder
      title="服务器列表"
      description="管理已接入的游戏服务器与插件实例。当前阶段仅提供页面结构。"
      planned={['服务器注册、编辑与下线', '服务器与插件版本信息', '服务器数据上报标识（API Key）绑定']}
    />
  );
}
