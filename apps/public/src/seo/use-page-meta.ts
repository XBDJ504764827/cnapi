import { useEffect } from 'react';

import { env } from '@/lib/env';

interface PageMetaOptions {
  title: string;
  description: string;
  path: string;
  type?: 'website' | 'profile' | 'article';
  robots?: string;
}

function setMeta(attribute: 'name' | 'property', key: string, content: string) {
  let element = document.head.querySelector<HTMLMetaElement>(`meta[${attribute}="${key}"]`);
  if (!element) {
    element = document.createElement('meta');
    element.setAttribute(attribute, key);
    document.head.appendChild(element);
  }
  element.setAttribute('content', content);
}

/**
 * Client-side metadata. Search engines that execute JavaScript pick this up;
 * static social crawlers prefer server-side/prerendered metadata, which is a
 * documented phase 5 improvement (see docs/public-portal/architecture.md).
 */
export function usePageMeta({ title, description, path, type = 'website', robots }: PageMetaOptions) {
  useEffect(() => {
    const fullTitle = `${title} · CNGOKZ`;
    const url = `${env.VITE_SITE_URL}${path}`;

    document.title = fullTitle;
    setMeta('name', 'description', description);
    setMeta('property', 'og:title', fullTitle);
    setMeta('property', 'og:description', description);
    setMeta('property', 'og:type', type);
    setMeta('property', 'og:url', url);
    setMeta('name', 'twitter:title', fullTitle);
    setMeta('name', 'twitter:description', description);
    if (robots) setMeta('name', 'robots', robots);

    let canonical = document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]');
    if (!canonical) {
      canonical = document.createElement('link');
      canonical.rel = 'canonical';
      document.head.appendChild(canonical);
    }
    canonical.href = url;
  }, [title, description, path, type, robots]);
}
