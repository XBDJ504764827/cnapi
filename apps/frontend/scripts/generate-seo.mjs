/**
 * Generates robots.txt and a core sitemap after `vite build`.
 * `/admin` is not indexed because it is the management UI.
 */
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const dist = path.resolve(here, '..', 'dist');
const siteUrl = (process.env.VITE_SITE_URL || 'https://test.cngokz.com').replace(/\/+$/, '');
const routes = ['/', '/leaderboards', '/players', '/maps', '/records', '/servers', '/compare', '/points'];

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
Disallow: /admin/

Sitemap: ${siteUrl}/sitemap.xml
`;

mkdirSync(dist, { recursive: true });
writeFileSync(path.join(dist, 'sitemap.xml'), sitemap);
writeFileSync(path.join(dist, 'robots.txt'), robots);
console.log(`[seo] wrote sitemap.xml and robots.txt for ${siteUrl}`);
