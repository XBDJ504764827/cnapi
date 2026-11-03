import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function ApiKeysPage() {
  return (
    <ModulePlaceholder
      title="API 密钥"
      description="为游戏服务器与第三方插件签发访问凭据。该模块为预留功能，尚未实现。"
      planned={['密钥创建、吊销与权限范围（scope）', '插件来源绑定与调用审计', '密钥轮换与过期策略']}
    />
  );
}
