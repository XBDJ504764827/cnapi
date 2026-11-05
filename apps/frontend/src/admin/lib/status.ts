import type { StatusTone } from '@/admin/types';

/** Maps an API status string (`up`, `down`, `healthy`, `degraded`) to a visual tone. */
export function toneFromStatus(status: string | null | undefined): StatusTone {
  switch (status) {
    case 'up':
    case 'healthy':
    case 'ready':
      return 'up';
    case 'down':
    case 'degraded':
    case 'not_ready':
      return 'down';
    default:
      return 'unknown';
  }
}

/** Maps the dependency names used by the API to display names. */
export function dependencyLabel(name: string): string {
  switch (name) {
    case 'postgresql':
      return 'PostgreSQL';
    case 'redis':
      return 'Redis';
    default:
      return name;
  }
}
