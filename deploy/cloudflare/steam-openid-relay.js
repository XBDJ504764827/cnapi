/**
 * Cloudflare Worker: Steam OpenID 中转
 * =====================================
 *
 * 生产服务器（境内）通常无法直连 steamcommunity.com。本 Worker 把
 * `https://<worker>/openid/*`（以及登录相关的少量路径）反向代理到
 * `https://steamcommunity.com/*`，让后端能够完成 OpenID 2.0 的
 * `check_authentication` 校验，必要时也承担浏览器侧的登录页中转。
 *
 * 用法（详见 docs/authentication.md 第 8 节）：
 *
 *   cd deploy/cloudflare
 *   npx wrangler deploy steam-openid-relay.js --name cngokz-steam-openid \
 *     --compatibility-date 2026-11-01
 *
 * 后端配置：
 *
 *   形态 A（浏览器可直连 Steam，仅服务器走中转）
 *     STEAM_OPENID_ENDPOINT=https://<worker>.workers.dev/openid/login
 *     # STEAM_OPENID_BROWSER_ENDPOINT 留空，默认仍指向 Steam 官方地址
 *
 *   形态 B（浏览器也不能直连 Steam，全中转）
 *     STEAM_OPENID_ENDPOINT=https://<worker>.workers.dev/openid/login
 *     STEAM_OPENID_BROWSER_ENDPOINT=https://<worker>.workers.dev/openid/login
 *
 * 安全建议：在 Cloudflare 控制台为本 Worker 加一条 WAF 规则，只允许
 * 生产服务器的出口 IP 访问 `/openid/login`，否则它会成为一个公开的 Steam 代理。
 */

const UPSTREAM = "https://steamcommunity.com";

/** 只代理 OpenID 与 Steam 登录所需的路径，其余一律 404。 */
const ALLOWED_PREFIXES = ["/openid/", "/login/", "/steamlogin", "/logout", "/actions/"];

export default {
  async fetch(request) {
    const url = new URL(request.url);

    if (!ALLOWED_PREFIXES.some((prefix) => url.pathname.startsWith(prefix))) {
      return new Response("Not found", { status: 404 });
    }

    const target = new URL(url.pathname + url.search, UPSTREAM);
    const init = {
      method: request.method,
      headers: upstreamHeaders(request),
      // 手动处理重定向：需要把指向 steamcommunity.com 的 Location 改写回 Worker 域名。
      redirect: "manual",
    };

    if (request.method !== "GET" && request.method !== "HEAD") {
      init.body = await request.arrayBuffer();
    }

    let upstream;
    try {
      upstream = await fetch(target, init);
    } catch (error) {
      // 网络失败时返回 502，后端会把校验失败映射为 auth_error=verification_failed。
      return new Response(`upstream error: ${error.message}`, { status: 502 });
    }

    const headers = new Headers(upstream.headers);
    rewriteLocation(headers, url);
    const cookies = rewriteSetCookies(upstream, url);
    if (cookies.length > 0) {
      headers.delete("set-cookie");
      for (const cookie of cookies) headers.append("set-cookie", cookie);
    }

    const contentType = headers.get("content-type") ?? "";
    if (contentType.includes("text/html")) {
      const html = rewriteHtml(await upstream.text(), url);
      headers.delete("content-length");
      return new Response(html, { status: upstream.status, headers });
    }

    return new Response(upstream.body, { status: upstream.status, headers });
  },
};

function upstreamHeaders(request) {
  const headers = new Headers(request.headers);
  // Host 必须由 fetch 依据目标地址生成，不能沿用 Worker 的域名。
  headers.delete("host");
  headers.delete("cf-connecting-ip");
  headers.set("x-forwarded-host", new URL(request.url).host);
  return headers;
}

/** `Location: https://steamcommunity.com/...` → `https://<worker>/...` */
function rewriteLocation(headers, url) {
  const location = headers.get("location");
  if (!location) return;
  headers.set("location", rewriteAbsoluteUrl(location, url));
}

function rewriteAbsoluteUrl(value, url) {
  return value
    .replaceAll("https://steamcommunity.com", url.origin)
    .replaceAll("http://steamcommunity.com", url.origin)
    .replaceAll("//steamcommunity.com", `//${url.host}`);
}

/** Set-Cookie 的 Domain 必须改写到 Worker 域名，否则浏览器会丢弃它。 */
function rewriteSetCookies(response, url) {
  const raw = typeof response.headers.getSetCookie === "function" ? response.headers.getSetCookie() : [];
  return raw.map((cookie) =>
    cookie
      .replace(/;\s*Domain=\.?steamcommunity\.com/gi, "")
      .replace(/;\s*Secure/gi, "; Secure"),
  );
}

/**
 * 形态 B 需要把页面里的绝对链接改回 Worker 域名，否则浏览器会直接跳去
 * steamcommunity.com。这是一个 best-effort 改写：Steam 改版后可能需要更新。
 */
function rewriteHtml(html, url) {
  return html
    .replaceAll("https://steamcommunity.com", url.origin)
    .replaceAll("http://steamcommunity.com", url.origin)
    .replaceAll("//steamcommunity.com", `//${url.host}`);
}
