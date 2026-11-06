# Steam 登录与管理员权限

本文档描述 CNGOKZAPI 的登录、会话与后台授权机制。**Steam 是唯一的身份来源**：没有邮箱密码、没有注册表单，也没有第二个登录入口。

- 后端实现：`apps/api/src/services/auth.rs`、`apps/api/src/handlers/auth.rs`、`apps/api/src/repositories/auth.rs`
- 数据库迁移：`apps/api/migrations/20261106000001_auth_steam_openid.sql`
- 前端实现：`apps/frontend/src/auth/session.ts`、`apps/frontend/src/admin/components/auth/require-admin.tsx`

---

## 1. 设计目标

| 目标 | 实现方式 |
| --- | --- |
| 只有 Steam 登录 | OpenID 2.0 `checkid_setup`，无本地账号密码 |
| 管理后台只有管理员能进 | `account_roles.role = 'admin'`；`/admin` 路由守卫 + 每个 `/api/v1/admin/*` 后端校验 |
| 普通玩家无法访问后台 | 已登录但非管理员 → 前端 403 页面，后端 403 |
| 生产环境无法直连 Steam | `STEAM_OPENID_ENDPOINT` 可指向 Cloudflare 中转，见第 8 节 |
| 不在浏览器里存密钥 | 会话是不透明的 HttpOnly Cookie，数据库只保存 SHA-256 摘要 |

## 2. 登录流程

```text
浏览器                        API                          Steam
  │  GET /api/v1/auth/steam/start?redirect=/admin/dashboard
  │──────────────────────────►│
  │                           │ 生成一次性 state（10 分钟有效，落库）
  │  303 → Steam OpenID 端点  │
  │◄──────────────────────────│
  │  用户输入 Steam 凭据 / 已登录
  │──────────────────────────────────────────────────────────►│
  │  303 → /api/v1/auth/steam/callback?state=…&openid.*
  │◄──────────────────────────────────────────────────────────│
  │──────────────────────────►│
  │                           │ 校验 state（原子消费，只能成功一次）
  │                           │ 校验 openid.return_to / op_endpoint / claimed_id
  │                           │ POST openid.mode=check_authentication ──►│
  │                           │◄── "is_valid:true" ─────────────────────│
  │                           │ 建账号、同步 Steam 资料、建会话
  │  303 → redirect 目标       │
  │  Set-Cookie: cngokz_session=…; HttpOnly; SameSite=Lax
  │◄──────────────────────────│
```

关键点：

- **`state` 单次使用**：`UPDATE … WHERE consumed_at IS NULL … RETURNING` 原子消费，重放返回 `auth_error=invalid_state`。
- **`openid.return_to` 精确匹配**：必须等于 `PUBLIC_BASE_URL + /api/v1/auth/steam/callback?state=<本次 state>`，否则拒绝。
- **`openid.op_endpoint` 校验**：必须等于 `STEAM_OPENID_OP_ENDPOINT`（默认 Steam 官方端点）。即使校验请求走 Cloudflare 中转，这条仍然指向 Steam 自身。
- **`claimed_id` ↔ `identity` 必须一致**，且形如 `https://steamcommunity.com/openid/id/<17–20 位数字>`。
- **签名验证交给 Steam**（OpenID "dumb mode"）：后端把收到的断言原样 POST 回身份提供方，只接受 `is_valid:true`。后端不实现 RSA，也不保存 Steam 公钥。

## 3. 数据库模型

### `account_roles`

```sql
account_id BIGINT REFERENCES accounts(id) ON DELETE CASCADE,
role TEXT NOT NULL CHECK (role IN ('admin')),
granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
granted_by BIGINT REFERENCES accounts(id) ON DELETE SET NULL,
PRIMARY KEY (account_id, role)
```

角色独立成表而不是 `accounts.is_admin`，后续增加 `moderator` 等角色不需要破坏性迁移。授权与账号解耦：**撤销角色后，已存在的会话在下一次请求就会失效**（每次请求都实时计算 `is_admin`）。

### `auth_sessions`

```sql
id UUID PRIMARY KEY,
account_id BIGINT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
token_hash BYTEA NOT NULL UNIQUE,  -- SHA-256(token)，长度固定 32
created_at, last_seen_at, expires_at, revoked_at, user_agent, ip
```

- Cookie 里是 256 位随机 token（64 位十六进制），数据库里只有摘要，**拖库无法重放成登录态**。
- `expires_at` 默认 30 天（`SESSION_TTL_HOURS=720`）。
- `revoked_at` 在退出登录时写入，立即失效。
- `last_seen_at` 最多每 5 分钟更新一次，避免每请求一次写。
- 过期 7 天以上的会话在登录时顺手清理。

### `steam_login_states`

`state` 主键 + `redirect_path`（登录后回到的同源路径）+ `expires_at` + `consumed_at`。`redirect_path` 上有 `CHECK` 约束，只允许 `/` 开头且不是 `//`。

### `player_profiles.avatar_url`

新增可空列，登录时若配置了 `STEAM_API_KEY` 则从 Steam Web API 刷新头像；没有 API Key 时保持 `NULL`，前端退化为首字母头像。

## 4. API

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/api/v1/auth/steam/start?redirect=/admin/dashboard` | 303 跳转到 Steam；`redirect` 必须是同源路径 |
| GET | `/api/v1/auth/steam/callback` | OpenID 回调：校验断言、建会话、303 回站点 |
| GET | `/api/v1/auth/me` | 当前会话；匿名返回 `200 { authenticated: false }`（不制造 401 噪音） |
| POST | `/api/v1/auth/logout` | 撤销会话 + `Max-Age=0` 清 Cookie，幂等 |

`/auth/me` 成功示例：

```json
{
  "success": true,
  "data": {
    "authenticated": true,
    "user": {
      "account_id": 1,
      "steam_id64": "76561198000000901",
      "display_name": "Player 0901",
      "avatar_url": null,
      "is_admin": true,
      "roles": ["admin"],
      "steam_profile_url": "https://steamcommunity.com/profiles/76561198000000901"
    },
    "expires_at": "2026-12-06T10:55:31.9Z"
  },
  "message": "ok"
}
```

失败回调不会渲染 JSON，而是 `303` 回站点并带上错误码：

| `auth_error` | 含义 |
| --- | --- |
| `invalid_state` | state 缺失、过期或已被使用（重放/超时） |
| `cancelled` | 用户在 Steam 页面取消 |
| `invalid_assertion` | 断言字段校验失败（`return_to`/`op_endpoint`/`claimed_id`） |
| `verification_failed` | 无法与 Steam 完成 `check_authentication` |
| `login_failed` | 建账号/建会话等内部错误 |

前端在 `apps/frontend/src/auth/session.ts` 中把这些码翻译成中文提示。

## 5. 授权规则

### 管理接口 `/api/v1/admin/*`

统一走 `services::auth::require_admin`：

1. 请求头带正确的 `X-Maintenance-Token` → 通过（供 CI/脚本/监控使用，令牌值只存在于服务端环境变量）；
2. 否则读取会话 Cookie → 未登录返回 **401**；
3. 已登录但 `account_roles` 里没有 `admin` → **403**。

> 前端产物中不再包含任何维护令牌：`VITE_MAINTENANCE_TOKEN` 已移除，浏览器只依赖 Cookie。

### 前端路由守卫

`RequireAdmin` 包裹整棵 `/admin` 路由树（`/admin/login` 除外）：

| 状态 | 表现 |
| --- | --- |
| 会话加载中 | 骨架屏（不会闪一下登录页） |
| 未登录 | `Navigate` 到 `/admin/login?next=<原路径>` |
| 已登录、非管理员 | 403 页面，显示当前账号与 SteamID64 |
| 管理员 | 渲染控制台 |

`next` 参数只接受 `/admin` 开头的路径，避免开放重定向。

## 6. 配置项

| 变量 | 默认值 | 说明 |
| --- | --- | --- |
| `PUBLIC_BASE_URL` | `http://localhost:5173` | 站点对外源；用于 OpenID `realm`、`return_to` 与登录后跳转。生产必须 `https://` |
| `STEAM_OPENID_ENDPOINT` | `https://steamcommunity.com/openid/login` | **服务端**校验断言用的端点（可指向 Cloudflare 中转） |
| `STEAM_OPENID_BROWSER_ENDPOINT` | 同 `STEAM_OPENID_ENDPOINT` | **浏览器**跳转用的端点；浏览器能直连 Steam 而服务器不能时用默认值 |
| `STEAM_OPENID_OP_ENDPOINT` | `https://steamcommunity.com/openid/login` | 断言中 `openid.op_endpoint` 的期望值，通常保持 Steam 官方地址 |
| `STEAM_OPENID_CLAIMED_ID_PREFIX` | `https://steamcommunity.com/openid/id/` | SteamID64 声明前缀，必须以 `https://` 开头并以 `/` 结尾 |
| `STEAM_API_KEY` | 空 | 可选 Steam Web API Key；配置后登录时同步昵称与头像 |
| `STEAM_API_BASE_URL` | `https://api.steampowered.com` | 可指向中转（生产服务器访问不到 Steam API 时） |
| `SESSION_COOKIE_NAME` | `cngokz_session` | 只允许 `[A-Za-z0-9_-]{1,64}` |
| `SESSION_TTL_HOURS` | `720` | 会话有效期（1–8760 小时） |
| `SESSION_COOKIE_SECURE` | 生产为 `true` | 走 HTTPS 时必须为 `true` |
| `SESSION_COOKIE_DOMAIN` | 空 | 一般留空（同源部署）；前导 `.` 会被去掉 |
| `ADMIN_STEAM_IDS` | 空 | 逗号分隔的 SteamID64；这些账号登录时自动获得 `admin` 角色（引导用途） |
| `MAINTENANCE_TOKEN` | 空 | 自动化令牌，与管理会话等效 |

本地开发最小配置（`.env`）：

```bash
PUBLIC_BASE_URL=http://localhost:5173
STEAM_OPENID_ENDPOINT=https://steamcommunity.com/openid/login
ADMIN_STEAM_IDS=7656119xxxxxxxxxx
# 可选：同步昵称与头像
# STEAM_API_KEY=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

开发时浏览器访问 `http://localhost:5173`，Vite 把 `/api` 代理到 `http://localhost:8080`（`VITE_API_PROXY_TARGET` 可改），因此 `return_to` 也落在 5173，Cookie 天然同源。

## 7. 管理员引导与撤销

**首次引导**：把 SteamID64 写进 `ADMIN_STEAM_IDS`，重启 API，该账号登录一次即写入 `account_roles`。之后可以从环境变量里删除，角色不会自动回收。

**授予/撤销**（需要在服务器上执行 SQL）：

```sql
-- 授予
INSERT INTO account_roles (account_id, role)
SELECT id, 'admin' FROM accounts WHERE steam_id64 = 7656119xxxxxxxxxx
ON CONFLICT DO NOTHING;

-- 撤销：立刻生效，无需等会话过期
DELETE FROM account_roles
WHERE account_id = (SELECT id FROM accounts WHERE steam_id64 = 7656119xxxxxxxxxx)
  AND role = 'admin';
```

**强制下线某个会话**：

```sql
UPDATE auth_sessions SET revoked_at = now()
WHERE account_id = (SELECT id FROM accounts WHERE steam_id64 = 7656119xxxxxxxxxx)
  AND revoked_at IS NULL;
```

## 8. 生产环境：Cloudflare 中转

### 8.1 为什么需要

生产服务器（境内）通常无法访问 `steamcommunity.com`。OpenID 的校验是**服务器主动发出的 POST**，因此至少需要一条能出去的通路。Cloudflare Worker 提供固定域名、TLS 与就近回源，是最低成本方案。

两种部署形态：

| 形态 | 浏览器能直连 Steam？ | 配置 |
| --- | --- | --- |
| A（推荐，改动最小） | 能 | `STEAM_OPENID_ENDPOINT=https://<worker>/openid/login`，`STEAM_OPENID_BROWSER_ENDPOINT` 留空（默认 Steam 官方地址） |
| B（全中转） | 不能 | 两个变量都指向 `https://<worker>/openid/login`，Worker 负责改写 `Location` 与页面内的绝对链接 |

形态 A 下，用户浏览器直接打开 `steamcommunity.com` 登录，只有服务器的校验请求经过 Cloudflare；`openid.op_endpoint` 仍然是 Steam 官方地址，校验逻辑不需要任何改动。

### 8.2 Worker

参考实现见 [`deploy/cloudflare/steam-openid-relay.js`](../deploy/cloudflare/steam-openid-relay.js)。

```bash
cd deploy/cloudflare
npx wrangler deploy steam-openid-relay.js --name cngokz-steam-openid \
  --compatibility-date 2026-11-01
```

建议再加一条缓存/防火墙规则：只允许你的服务器出口 IP 访问 `/openid/login`（形态 A），避免中转被当成免费代理。

### 8.3 验证

```bash
# 服务器上直接验证出网能力（应当返回 200/302，而不是超时）
curl -sS -o /dev/null -w '%{http_code}\n' \
  -X POST 'https://<worker>/openid/login' \
  -H 'Content-Type: application/x-www-form-urlencoded' \
  --data 'openid.mode=check_authentication&openid.ns=http://specs.openid.net/auth/2.0'

# 完整流程：浏览器打开 /admin/login，用管理员 Steam 账号登录
```

### 8.4 生产检查清单

- [ ] `PUBLIC_BASE_URL=https://test.cngokz.com`（必须 https）
- [ ] `STEAM_OPENID_ENDPOINT` 指向 Worker，并在服务器上用上面的 curl 验证过
- [ ] `ADMIN_STEAM_IDS` 至少包含一名管理员
- [ ] `STEAM_API_KEY` 已配置（否则昵称显示为 `Player xxxx`，头像为空）
- [ ] Nginx `/api/` 透传 `Cookie` 与 `Set-Cookie`（现有配置默认透传，未做拦截）
- [ ] OpenResty 未对 `/api/v1/auth/` 开启缓存

## 9. 安全设计

| 风险 | 处置 |
| --- | --- |
| CSRF（伪造回调） | 一次性 `state` + `return_to` 精确匹配 + `SameSite=Lax` |
| 开放重定向 | `redirect`/`next` 只接受同源路径，拒绝 `//`、`\`、控制字符 |
| 会话泄露 | HttpOnly + Secure（生产）+ 仅存 SHA-256 摘要 |
| 拖库重放 | 数据库无明文 token |
| 会话固定 | 登录总是新建 token，不复用任何既有值 |
| 令牌时序侧信道 | 维护令牌比较使用定长比较 |
| 伪造断言 | 交回 Steam `check_authentication`，本地只做结构校验 |
| 越权访问后台 | 前端守卫 + 每个 `/api/v1/admin/*` 后端 `require_admin`，双重校验 |
| 日志泄露 | 不记录 token、Cookie、Steam API Key；`AppConfig` 的 `Debug` 对密钥做 `***` 脱敏 |

## 10. 测试

| 层次 | 位置 | 覆盖 |
| --- | --- | --- |
| 单元 | `apps/api/src/services/auth.rs` | 重定向路径校验、`return_to` 构造、断言校验、SteamID64 提取、Cookie 属性、常量时间比较 |
| 单元 | `apps/api/src/handlers/auth.rs` | 查询串解析（保留重复键）、`X-Forwarded-For` |
| 集成 | `apps/api/tests/auth_flow.rs` | 完整 round trip（本地 mock OP）、管理员/非管理员、重放、伪造断言、退出登录、维护令牌 |
| 前端 | `apps/frontend/src/auth/session.test.ts` | 登录 URL、错误码映射、头像首字母 |
| 前端 | `apps/frontend/src/admin/components/auth/require-admin.test.tsx` | 未登录跳转、非管理员 403、管理员放行、加载态不闪登录页 |

本地跑集成测试（需要真实 PostgreSQL）：

```bash
DATABASE_URL=postgres://user@127.0.0.1:5432/cngokzapi cargo test -p cngokz-api --test auth_flow
```

`auth_flow` 会自行创建临时数据库并应用迁移，结束后不残留。它用本地 axum mock 代替 Steam，**不访问外网**。

## 11. 已知限制

- 没有"记住我"之外的会话管理页面（无法在 UI 里查看/踢掉其它设备）。
- 没有登录频率限制（依赖外层 Nginx/Cloudflare 限流）；如需可在 `/auth/steam/start` 前加一层。
- Steam Web API 调用失败不影响登录，只是昵称/头像保持旧值。
- 形态 B（浏览器也不能直连 Steam）的 Worker 需要改写页面内绝对链接，属于 best-effort，Steam 改版时可能需要调整。
- 账号删除（软删除）后，同一 SteamID64 再次登录会创建新账号记录（由 `accounts` 的部分唯一索引保证）。
