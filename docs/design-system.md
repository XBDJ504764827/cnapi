# 设计系统（Design System）

第三阶段把管理后台从"能用的脚手架"升级为统一的专业管理平台。本文档定义颜色、字体、间距、组件和交互规范；所有页面必须复用这些 token 与组件，不在页面里写一次性样式。

- 实现位置：`apps/web/src/styles/index.css`（token）+ `apps/web/src/components/ui/*`（组件）
- 主题默认：**深色**；可选浅色 / 跟随系统，偏好持久化在 `localStorage`（key `cngokz-ui`）
- 设计参考：Linear（深色层次与交互）、Vercel（信息架构）、Grafana（状态展示）、Cloudflare（基础设施管理）

## 1. 设计原则

1. **层次靠表面与边框**，不靠发光、玻璃拟态或大面积渐变。
2. **深灰而非纯黑**：背景 `oklch(0.168)`，表面依次提亮。
3. **一个强调色**：冷蓝 `primary` 只用于主操作、选中与关键数据。
4. **语义色只表达状态**：成功/警告/危险/信息各有独立前景色，不混用。
5. **数字对齐**：统计、时间、延迟使用 `tabular-nums`。
6. **诚实状态**：没有接口就没有数据 — 统一使用空状态 + "接口未接入"说明，绝不伪造统计。
7. **微动效**：150–220ms 的淡入/缩放/滑动，尊重 `prefers-reduced-motion`。

## 2. 颜色 Token

所有颜色定义在 `:root`（浅色）与 `.dark`（深色），并通过 `@theme inline` 映射为 Tailwind 工具类（`bg-surface`、`text-muted-foreground` 等）。

> Token 与基础层已抽取到 `packages/ui/src/design-system.css`（`@cngokz/ui/styles.css`），管理后台与公开门户共享同一套变量；公开门户的布局/组件差异见 [public-portal/design-system.md](public-portal/design-system.md)。

### 语义角色

| Token | 用途 | 深色 | 浅色 |
| --- | --- | --- | --- |
| `--background` | 页面底色 | `oklch(0.168 0.004 260)` | `oklch(0.968 0.002 255)` |
| `--surface` | 卡片/面板 | `oklch(0.195 0.005 260)` | `oklch(1 0 0)` |
| `--surface-elevated` | 输入框、次级面板 | `oklch(0.218 0.006 260)` | `oklch(0.988 0.002 255)` |
| `--surface-hover` | hover 背景 | `oklch(0.232 0.006 260)` | `oklch(0.955 0.004 255)` |
| `--foreground` | 主文本 | `oklch(0.955 0.003 250)` | `oklch(0.24 0.012 260)` |
| `--muted-foreground` | 次要文本 | `oklch(0.655 0.009 255)` | `oklch(0.52 0.012 260)` |
| `--border` | 常规边框 | `oklch(0.285 0.006 260)` | `oklch(0.9 0.004 260)` |
| `--border-strong` | 强调分隔/滚动条 | `oklch(0.34 0.008 260)` | `oklch(0.84 0.006 260)` |
| `--primary` | 主操作/选中 | `oklch(0.67 0.15 255)` | `oklch(0.55 0.16 255)` |
| `--success` | 正常/通过 | `oklch(0.72 0.15 155)` | `oklch(0.55 0.14 155)` |
| `--warning` | 降级/待审核 | `oklch(0.79 0.14 80)` | `oklch(0.66 0.14 75)` |
| `--danger`（别名 `--destructive`） | 不可用/拒绝 | `oklch(0.66 0.19 25)` | `oklch(0.56 0.19 25)` |
| `--info` | 提示/GET | `oklch(0.72 0.12 235)` | `oklch(0.55 0.12 235)` |
| `--sidebar*` | 侧边栏（背景/前景/选中/边框） | 深一档表面 | 浅一档表面 |

### 使用规则

- 主操作：`bg-primary text-primary-foreground`；每个页面最多一个主按钮。
- 状态徽章：`Badge` 的 `success / warning / danger / info` 变体（12% 背景 + 25% 边框 + 语义前景）。
- 禁止用 `success` 表达普通强调，也禁止用 `primary` 表达"已通过"。
- 表格 hover 使用 `bg-surface-hover/60`，选中行使用 `bg-surface-hover`。

## 3. 字体系统

| 角色 | 字体/字号/字重 | 说明 |
| --- | --- | --- |
| 全局 | `Inter Variable` → 系统中文字体回退 | `@fontsource-variable/inter` 自托管，无运行时外链 |
| 页面标题 | 22px / 600 / tracking-tight | `PageHeader` |
| 页面描述 | 13px / 400 / muted | |
| 卡片标题 | 14px / 600 | `CardTitle` |
| 卡片描述 | 12px / 400 / muted | `CardDescription` |
| 正文/表格 | 13px / 400 | 表头 11px / 500 / uppercase / letter-spacing 0.045em |
| 辅助信息 | 11–12px / muted | 时间、ID、提示 |
| 数字/统计 | 26px / 600 / `tabular-nums` | 统计卡片 |
| 代码/ID | `ui-monospace` 12px | SteamID64、路径、Key 前缀 |

规则：

- 不使用 4 种以上字号层级；不随意加粗。
- 数字列右对齐或等宽，时间使用 `font-mono`。
- 中文与英文之间保留自然空格；标点统一中文全角。

## 4. 间距、尺寸与圆角

| 项 | 规范 |
| --- | --- |
| 页面最大宽度 | `1500px`，居中 |
| 页面内边距 | 16px（手机）/ 24px（平板）/ 32px（桌面） |
| 区块间距 | 20–24px（`space-y-5` / `space-y-6`） |
| 卡片内边距 | 水平 20px，头 16/12，主体 20/16 |
| 卡片间距 | 12–16px（`gap-3` / `gap-4`） |
| 输入框高度 | 36px（`h-9`），小号 28px（`h-7`） |
| 按钮高度 | 28 / 32 / 36 / 40px（xs/sm/default/lg） |
| 表格行高 | 表头 36px，数据行约 44px |
| 圆角 | 基础 8px；输入/按钮 6px；卡片 8px；弹窗 12px；徽章 4px |
| 侧边栏 | 展开 240px，折叠 64px；顶部 56px |
| 顶栏 | 56px sticky |

## 5. 组件规范

### 5.1 基础组件（`components/ui`）

| 组件 | 变体/尺寸 | 关键规则 |
| --- | --- | --- |
| `Button` | default / secondary / outline / ghost / destructive / link；xs–lg + icon；`loading` | 主按钮唯一；图标 16px；`asChild` 用于链接 |
| `Input` | 36px 高；错误态 `aria-invalid` | 焦点环 20% primary |
| `SearchInput` | 图标 + 清除 + 快捷键提示 | 受控；清除按钮有 aria-label |
| `Select` | 36/28px；Radix | 选项带勾选指示 |
| `Dialog` | 最大 512px；遮罩 60% + 2px 模糊 | 必须有 Title；关闭按钮含 sr-only |
| `DropdownMenu` | 菜单/单选 | 选中项带 Check；禁用项降低透明度 |
| `Tooltip` | 150ms 延迟 | 仅补充信息，不承载关键内容 |
| `Tabs` | 下划线样式 | 用于详情页/榜单切换 |
| `Badge` | default/secondary/primary/outline/success/warning/danger/info | 11px；12% 背景 + 25% 边框 |
| `Card` | header/action/content/footer | 边框 + 极浅阴影，无装饰 |
| `Table` / `DataTable` | 横向滚动；骨架行；空/错误态 | 空数据时不渲染宽表头，避免移动端裁切 |
| `Pagination` | 页码窗口 + 省略号 + 每页条数 | 无数据时上一页/下一页禁用 |
| `Skeleton` | 脉冲 | 仅用于真实加载中 |
| `StatusIndicator` | up/down/unknown + 可选脉冲 | 绿/红/灰语义点 |
| `Callout` | info/success/warning/danger/neutral | 用于"接口未接入"、降级、错误 |
| `Kbd` | 10px keycap | 快捷键提示 |

### 5.2 组合组件

| 组件 | 位置 | 说明 |
| --- | --- | --- |
| `PageHeader` | `components/common` | 标题、描述、meta、操作 |
| `ListPanel` / `FilterBar` | `components/common` | 列表页统一卡片、工具栏、分页 |
| `EmptyState` / `ErrorState` / `PageLoading` | `components/common` | 三态统一 |
| `CodeBlock` | `components/common` | API 示例 + 真实复制 |
| `ServerCard` | `components/servers` | 服务器卡片视图 |
| `StatCard` | `components/dashboard` | 统计占位/真实值 |
| `ServiceStatusRow` / `DependencyStatusCard` | `components/dashboard` | 真实健康状态 |

## 6. 布局与响应式

```text
┌────────────┬─────────────────────────────────────────┐
│ Sidebar    │ Header（面包屑 / 搜索 / 通知 / 主题 / 用户） │
│ 240 / 64px ├─────────────────────────────────────────┤
│            │ Main（max 1500px，20–32px 内边距）         │
│ 分组导航    │ PageHeader / 内容区块 / 列表 / 分页        │
│ 系统状态    ├─────────────────────────────────────────┤
│            │ Footer（产品说明 / 阶段）                  │
└────────────┴─────────────────────────────────────────┘
```

| 断点 | 布局 |
| --- | --- |
| ≥1536（1920） | 完整侧边栏 + 4 列统计 + 3 列内容 |
| 1280–1535 | 完整侧边栏 + 4 列统计 |
| 1024–1279 | 侧边栏可见（可折叠）+ 2 列统计 |
| 640–1023 | 侧边栏收进抽屉（汉堡菜单）+ 2 列统计 |
| <640（375） | 单列统计/卡片；表格横向滚动；空状态全宽；工具栏换行 |

移动端表格策略：容器 `overflow-x-auto` + `min-width`；空状态在滚动容器之外渲染，避免被裁切。详情页顶栏隐藏分组名，只保留当前页面。

## 7. 状态与反馈

| 状态 | 表现 |
| --- | --- |
| Loading | 骨架屏（表格 5 行、卡片骨架）；不使用转圈遮罩 |
| Empty | 图标 + 标题 + 说明 + 可选操作；明确"接口未接入" |
| Error | `Callout` 危险样式 + 重试按钮；展示来自 API 的安全错误消息 |
| Degraded | 黄色 Callout + 侧边栏状态点变红/灰 |
| Success action | 本阶段没有写操作，因此不展示成功 toast |

**不伪造数据**：没有接口的统计显示 `—` + "接口未接入"；服务器/请求/审计等页面只展示空状态，不生成模拟行。

## 8. 动效

- 页面进入：`page-enter`（180ms 淡入+上移 4px）。
- 弹窗/下拉：`zoom-in`（140ms）。
- 移动抽屉：`slide-in-left`（220ms）。
- 主题切换：图标旋转/缩放 150ms。
- `prefers-reduced-motion: reduce` 下全部降为 0.01ms。

## 9. 可访问性

- 所有图标按钮有 `aria-label`；纯装饰图标 `aria-hidden`。
- 弹窗/抽屉包含 `DialogTitle`；表单使用 `Label htmlFor`。
- 焦点可见：2px `ring`；跳转主内容链接在聚焦时显示。
- 状态不只靠颜色：`StatusIndicator` 同时输出文字。
- 表格空/错误态不隐藏关键说明；分页按钮有 `aria-label`/`aria-current`。

## 10. 文件索引

| 内容 | 路径 |
| --- | --- |
| Token / 字体 / 动效 / 基础层 | `apps/web/src/styles/index.css` |
| 基础组件 | `apps/web/src/components/ui/` |
| 布局（Sidebar/Header/CommandMenu） | `apps/web/src/components/layout/` |
| 应用外壳 | `apps/web/src/layouts/app-layout.tsx` |
| 页面 | `apps/web/src/pages/` |
| 路由 | `apps/web/src/router/index.tsx` |
| UI 状态（主题/侧边栏） | `apps/web/src/stores/ui-store.ts` |
| 设计参考实现验证 | `docs/changelog.md`（第三阶段记录 + Playwright 结果） |
