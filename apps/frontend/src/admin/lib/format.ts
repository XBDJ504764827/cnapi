/** Human readable uptime, e.g. `2 天 3 小时 12 分 5 秒`. */
export function formatUptime(totalSeconds: number): string {
  const seconds = Math.max(0, Math.floor(totalSeconds));
  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);
  const remainingSeconds = seconds % 60;

  const parts: string[] = [];
  if (days > 0) parts.push(`${days} 天`);
  if (days > 0 || hours > 0) parts.push(`${hours} 小时`);
  if (days > 0 || hours > 0 || minutes > 0) parts.push(`${minutes} 分`);
  parts.push(`${remainingSeconds} 秒`);

  return parts.join(' ');
}

const dateTimeFormatter = new Intl.DateTimeFormat('zh-CN', {
  dateStyle: 'medium',
  timeStyle: 'medium',
});

/** Formats an ISO timestamp in the viewer's locale/time zone. */
export function formatDateTime(value: string | Date | null | undefined): string {
  if (!value) return '—';
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return '—';
  return dateTimeFormatter.format(date);
}

/** Relative time for event feeds, e.g. `3 分钟前`. */
export function formatRelativeTime(value: string | Date | null | undefined, now: Date = new Date()): string {
  if (!value) return '—';
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return '—';

  const diffSeconds = Math.round((now.getTime() - date.getTime()) / 1000);
  if (diffSeconds < 60) return '刚刚';
  if (diffSeconds < 3_600) return `${Math.floor(diffSeconds / 60)} 分钟前`;
  if (diffSeconds < 86_400) return `${Math.floor(diffSeconds / 3_600)} 小时前`;
  if (diffSeconds < 2_592_000) return `${Math.floor(diffSeconds / 86_400)} 天前`;
  return formatDateTime(date);
}

/** Latency in milliseconds, `—` when unknown. */
export function formatLatency(latencyMs: number | null | undefined): string {
  if (latencyMs === null || latencyMs === undefined) return '—';
  return `${latencyMs} ms`;
}

/** Formats a number with locale grouping; `—` for missing values. */
export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—';
  return value.toLocaleString('zh-CN');
}
