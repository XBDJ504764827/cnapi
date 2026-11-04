/**
 * Minimal production static server for the built web app.
 *
 * Responsibilities:
 * - serve `dist/` with long lived caching for hashed assets
 * - SPA fallback to `index.html`
 * - reverse proxy `/api`, `/health`, `/swagger-ui` and `/api-docs` to the API
 *   container so the browser only talks to one origin (no CORS, no build time
 *   host configuration)
 * - expose `/healthz` for container health checks
 *
 * No runtime dependencies on purpose: the container image only needs Node.
 */
import http from 'node:http';
import { createReadStream, existsSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(here, '..', 'dist');
const port = Number(process.env.WEB_PORT ?? 5173);
const upstream = new URL(process.env.API_UPSTREAM ?? 'http://localhost:8080');

const PROXY_PREFIXES = ['/api', '/health', '/swagger-ui', '/api-docs'];
const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.webp': 'image/webp',
  '.ico': 'image/x-icon',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
  '.map': 'application/json; charset=utf-8',
};

const server = http.createServer((request, response) => {
  const url = new URL(request.url ?? '/', `http://${request.headers.host ?? 'localhost'}`);

  if (url.pathname === '/healthz') {
    response.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
    response.end(JSON.stringify({ status: 'ok' }));
    return;
  }

  if (PROXY_PREFIXES.some((prefix) => url.pathname === prefix || url.pathname.startsWith(`${prefix}/`))) {
    proxyToApi(request, response, url);
    return;
  }

  // The admin app is served under /admin/ (public portal gateway proxies with
  // the prefix preserved). Strip it before resolving files from dist/.
  const pathname =
    url.pathname === '/admin' || url.pathname.startsWith('/admin/')
      ? url.pathname.slice('/admin'.length) || '/'
      : url.pathname;

  if (url.pathname === '/admin') {
    response.writeHead(308, { location: '/admin/' });
    response.end();
    return;
  }

  serveStatic(response, pathname);
});

function serveStatic(response, pathname) {
  let decoded;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    response.writeHead(400).end('Bad Request');
    return;
  }
  // path.join collapses '..' segments; reject anything that escapes distDir.
  const resolved = path.resolve(path.join(distDir, decoded));
  const relative = path.relative(distDir, resolved);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    response.writeHead(403).end('Forbidden');
    return;
  }

  if (!resolved.startsWith(distDir)) {
    response.writeHead(403).end('Forbidden');
    return;
  }

  if (existsSync(resolved) && statSync(resolved).isFile()) {
    sendFile(response, resolved, '/' + relative.replaceAll(path.sep, '/'));
    return;
  }

  // SPA fallback: unknown paths are handled by the React router.
  const indexFile = path.join(distDir, 'index.html');
  if (!existsSync(indexFile)) {
    response.writeHead(503, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Web build is missing. Run `pnpm --filter @cngokz/web build`.');
    return;
  }
  sendFile(response, indexFile, '/index.html');
}

function sendFile(response, filePath, requestPath) {
  const extension = path.extname(filePath).toLowerCase();
  const immutable = requestPath.startsWith('/assets/') && extension !== '.html';

  response.writeHead(200, {
    'content-type': MIME_TYPES[extension] ?? 'application/octet-stream',
    'cache-control': immutable ? 'public, max-age=31536000, immutable' : 'no-cache',
  });
  createReadStream(filePath).pipe(response);
}

function proxyToApi(request, response, url) {
  const targetPath = `${url.pathname}${url.search}`;

  const upstreamRequest = http.request(
    {
      protocol: upstream.protocol,
      hostname: upstream.hostname,
      port: upstream.port,
      method: request.method,
      path: targetPath,
      headers: { ...request.headers, host: upstream.host },
    },
    (upstreamResponse) => {
      response.writeHead(upstreamResponse.statusCode ?? 502, upstreamResponse.headers);
      upstreamResponse.pipe(response);
    },
  );

  upstreamRequest.on('error', (error) => {
    console.error(`[proxy] ${request.method} ${targetPath} failed: ${error.message}`);
    if (response.headersSent) {
      response.end();
      return;
    }
    response.writeHead(502, { 'content-type': 'application/json; charset=utf-8' });
    response.end(
      JSON.stringify({
        success: false,
        error: { code: 'upstream_unavailable', details: null },
        message: 'API upstream is unavailable',
      }),
    );
  });

  request.pipe(upstreamRequest);
}

server.listen(port, '0.0.0.0', () => {
  console.log(`[web] static server listening on http://0.0.0.0:${port}`);
  console.log(`[web] proxying ${PROXY_PREFIXES.join(', ')} to ${upstream.origin}`);
});

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => {
    console.log(`[web] received ${signal}, shutting down`);
    server.close(() => process.exit(0));
  });
}
