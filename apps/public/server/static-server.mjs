/**
 * Production static server for the public portal.
 * - serves the Vite build with SPA fallback
 * - proxies /api, /health, /swagger-ui and /api-docs to the API container
 * - exposes /healthz for container health checks
 */
import http from 'node:http';
import { createReadStream, existsSync, statSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const distDir = path.resolve(here, '..', 'dist');
const port = Number(process.env.PUBLIC_PORT ?? 5173);
const upstream = new URL(process.env.API_UPSTREAM ?? 'http://localhost:8080');
const adminUpstream = new URL(process.env.ADMIN_UPSTREAM ?? 'http://localhost:5174');
const PROXY_PREFIXES = ['/api', '/health', '/swagger-ui', '/api-docs'];
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.xml': 'application/xml; charset=utf-8',
  '.txt': 'text/plain; charset=utf-8',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.jpg': 'image/jpeg',
  '.webp': 'image/webp',
  '.ico': 'image/x-icon',
  '.woff': 'font/woff',
  '.woff2': 'font/woff2',
};

const server = http.createServer((request, response) => {
  const url = new URL(request.url ?? '/', `http://${request.headers.host ?? 'localhost'}`);

  if (url.pathname === '/healthz') {
    response.writeHead(200, { 'content-type': 'application/json; charset=utf-8' });
    response.end('{"status":"ok"}');
    return;
  }

  if (url.pathname === '/admin' || url.pathname.startsWith('/admin/')) {
    proxy(request, response, url, adminUpstream);
    return;
  }

  if (PROXY_PREFIXES.some((prefix) => url.pathname === prefix || url.pathname.startsWith(`${prefix}/`))) {
    proxy(request, response, url, upstream);
    return;
  }

  serve(response, url.pathname);
});

function serve(response, pathname) {
  let decoded;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    response.writeHead(400).end('Bad Request');
    return;
  }
  const resolved = path.resolve(path.join(distDir, decoded));
  const relative = path.relative(distDir, resolved);
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    response.writeHead(403).end('Forbidden');
    return;
  }
  if (existsSync(resolved) && statSync(resolved).isFile()) {
    send(response, resolved, '/' + relative.replaceAll(path.sep, '/'));
    return;
  }
  const index = path.join(distDir, 'index.html');
  if (!existsSync(index)) {
    response.writeHead(503, { 'content-type': 'text/plain; charset=utf-8' });
    response.end('Public portal build missing. Run `pnpm --filter @cngokz/public build`.');
    return;
  }
  send(response, index, '/index.html');
}

function send(response, file, requestPath) {
  const extension = path.extname(file).toLowerCase();
  const immutable = requestPath.startsWith('/assets/') && extension !== '.html';
  response.writeHead(200, {
    'content-type': MIME[extension] ?? 'application/octet-stream',
    'cache-control': immutable ? 'public, max-age=31536000, immutable' : 'no-cache',
  });
  createReadStream(file).pipe(response);
}

function proxy(request, response, url, target) {
  const targetPath = `${url.pathname}${url.search}`;
  const upstreamRequest = http.request(
    {
      protocol: target.protocol,
      hostname: target.hostname,
      port: target.port,
      method: request.method,
      path: targetPath,
      headers: { ...request.headers, host: target.host },
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
        message: 'API upstream unavailable',
      }),
    );
  });
  request.pipe(upstreamRequest);
}

server.listen(port, '0.0.0.0', () => {
  console.log(`[public] listening on http://0.0.0.0:${port}`);
  console.log(`[public] proxying ${PROXY_PREFIXES.join(', ')} to ${upstream.origin}`);
  console.log(`[public] proxying /admin to ${adminUpstream.origin}`);
});

for (const signal of ['SIGINT', 'SIGTERM']) {
  process.on(signal, () => server.close(() => process.exit(0)));
}
