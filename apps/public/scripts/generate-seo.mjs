/**
 * Generates robots.txt and a core sitemap after `vite build`.
 *
 * Player and map URLs are dynamic and would require a data snapshot at build
 * time; a dynamic sitemap endpoint is a documented phase 5 improvement.
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(here, '..', 'dist');
const siteUrl = (process.env.VITE_SITE_URL || 'http://localhost:5173').replace(/\/+$/, '');

const routes = ['/', '/leaderboards', '/players', '/maps', '/records', '/servers', '/compare'];

const sitemap = `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
${routes.map((route) => `  <url><loc>${siteUrl}${route}</loc></url>`).join('\n')}
</urlset>
`;

const robots = `User-agent: *
Allow: /
Disallow: /api/
Disallow: /api-docs/
Disallow: /swagger-ui/

Sitemap: ${siteUrl}/sitemap.xml
`;

mkdirSync(dist, { recursive: true });
writeFileSync(path.join(dist, 'sitemap.xml'), sitemap);
writeFileSync(path.join(dist, 'robots.txt'), robots);
console.log(`[seo] wrote sitemap.xml and robots.txt for ${siteUrl}`);
