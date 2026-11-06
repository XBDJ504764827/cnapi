/**
 * Static reference options used by filters and tables.
 *
 * Values mirror the seeded reference tables (`movement_modes`,
 * `movement_styles`, `jump_types`) and the TEXT CHECK constraints in
 * `apps/api/migrations/0002`–`0009`. They are display metadata only — real
 * data always comes from the API once the endpoints exist.
 */

export interface SelectOption {
  value: string;
  label: string;
}

export const movementModes: SelectOption[] = [
  { value: 'vanilla', label: 'Vanilla' },
  { value: 'classic', label: 'Classic' },
  { value: 'simple', label: 'Simple KZ' },
  { value: 'kztimer', label: 'KZTimer' },
];

export const movementStyles: SelectOption[] = [{ value: 'normal', label: 'Normal' }];

export const timeTypes: SelectOption[] = [
  { value: 'pro', label: 'PRO' },
  { value: 'nub', label: 'NUB' },
];

export const recordStatuses: SelectOption[] = [
  { value: 'pending', label: '待审核' },
  { value: 'approved', label: '已通过' },
  { value: 'rejected', label: '已拒绝' },
];

export const mapStatuses: SelectOption[] = [
  { value: 'in_testing', label: '测试中' },
  { value: 'approved', label: '已上线' },
  { value: 'disabled', label: '已停用' },
];

export const banStatuses: SelectOption[] = [
  { value: 'active', label: '生效中' },
  { value: 'revoked', label: '已撤销' },
];

export const banSources: SelectOption[] = [
  { value: 'admin', label: '管理员' },
  { value: 'server', label: '服务器' },
  { value: 'globalapi_import', label: 'GlobalAPI 导入' },
];

export const jumpTypes: SelectOption[] = [
  { value: 'long-jump', label: 'Long Jump' },
  { value: 'bhop', label: 'Bunnyhop' },
  { value: 'multi-bhop', label: 'Multi Bunnyhop' },
  { value: 'weird-jump', label: 'Weird Jump' },
  { value: 'ladder-jump', label: 'Ladder Jump' },
  { value: 'ladderhop', label: 'Ladderhop' },
  { value: 'jumpbug', label: 'Jumpbug' },
  { value: 'lowpre-bhop', label: 'Lowpre Bunnyhop' },
  { value: 'lowpre-weird-jump', label: 'Lowpre Weird Jump' },
  { value: 'fall', label: 'Fall' },
  { value: 'other', label: 'Other' },
  { value: 'invalid', label: 'Invalid' },
];

export const httpMethods: SelectOption[] = [
  { value: 'GET', label: 'GET' },
  { value: 'POST', label: 'POST' },
  { value: 'PATCH', label: 'PATCH' },
  { value: 'DELETE', label: 'DELETE' },
];

export const auditActions: SelectOption[] = [
  { value: 'record.review', label: '成绩审核' },
  { value: 'map.update', label: '地图更新' },
  { value: 'server.update', label: '服务器更新' },
  { value: 'ban.create', label: '创建封禁' },
  { value: 'ban.revoke', label: '解除封禁' },
  { value: 'key.create', label: '创建密钥' },
  { value: 'key.revoke', label: '吊销密钥' },
];

export const pageSizeOptions = [10, 20, 50, 100];
