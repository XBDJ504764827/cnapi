import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function SettingsPage() {
  return (
    <ModulePlaceholder
      title="系统设置"
      description="维护平台级配置项（app_settings 表已就绪，读写接口尚未实现）。"
      planned={['配置项分组展示与编辑', '修改审计与回滚', '敏感配置的掩码显示']}
    />
  );
}
