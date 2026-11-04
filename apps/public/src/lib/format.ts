import type { TimeType } from '@cngokz/shared-types';

const dateTime = new Intl.DateTimeFormat('zh-CN', { dateStyle: 'medium', timeStyle: 'short' });

export function formatDateTime(value: string | Date | null | undefined): string {
  if (!value) return '—';
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return '—';
  return dateTime.format(date);
}

export function formatRelativeTime(value: string | Date | null | undefined, now = new Date()): string {
  if (!value) return '—';
  const date = value instanceof Date ? value : new Date(value);
  if (Number.isNaN(date.getTime())) return '—';
  const seconds = Math.round((now.getTime() - date.getTime()) / 1000);
  if (seconds < 60) return '刚刚';
  if (seconds < 3600) return `${Math.floor(seconds / 60)} 分钟前`;
  if (seconds < 86_400) return `${Math.floor(seconds / 3600)} 小时前`;
  if (seconds < 2_592_000) return `${Math.floor(seconds / 86_400)} 天前`;
  return formatDateTime(date);
}

/** `12.345` — SQL-sourced decimal seconds string, never re-parsed as float. */
export function formatSeconds(seconds: string | null | undefined): string {
  if (!seconds) return '—';
  const value = seconds.split('.')[0] ?? seconds;
  const fraction = seconds.split('.')[1] ?? '000';
  const trimmed = fraction.slice(0, 3).padEnd(3, '0');
  return `${value}.${trimmed}`;
}

export function formatNumber(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—';
  return value.toLocaleString('zh-CN');
}

export function formatPercent(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—';
  return `${(value * 100).toFixed(1)}%`;
}

export function timeTypeLabel(value: TimeType | null | undefined): string {
  if (value === 'pro') return 'PRO';
  if (value === 'nub') return 'NUB';
  return '—';
}

export function tierLabel(tier: number | null | undefined): string {
  return tier ? `T${tier}` : '未评级';
}

export function modeLabel(code: string): string {
  const labels: Record<string, string> = {
    vanilla: 'Vanilla',
    simple: 'Simple KZ',
    kztimer: 'KZTimer',
    classic: 'Classic',
  };
  return labels[code] ?? code;
}

export function initials(name: string): string {
  const trimmed = name.trim();
  if (!trimmed) return '?';
  return trimmed.slice(0, 2).toUpperCase();
}

/** Formats an SQL decimal-seconds string to 3 fractional digits. */
export function formatSecondsPrecise(value: string | null | undefined): string {
  if (!value) return '—';
  const [integer = '0', fraction = '000'] = value.split('.');
  return `${integer}.${fraction.slice(0, 3).padEnd(3, '0')}`;
}
