import { ModulePlaceholder } from '@/components/common/module-placeholder';

export function ReplaysPage() {
  return (
    <ModulePlaceholder
      title="Replay"
      description="管理成绩回放文件的元数据与存储位置。当前阶段仅提供页面结构。"
      planned={['Replay 列表与成绩关联', '存储位置与文件完整性状态', '过期 Replay 清理策略']}
    />
  );
}
