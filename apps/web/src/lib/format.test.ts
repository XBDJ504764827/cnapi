import { describe, expect, it } from 'vitest';

import { formatDateTime, formatLatency, formatRelativeTime, formatUptime } from './format';

describe('formatUptime', () => {
  it('formats seconds, minutes, hours and days', () => {
    expect(formatUptime(0)).toBe('0 秒');
    expect(formatUptime(45)).toBe('45 秒');
    expect(formatUptime(90)).toBe('1 分 30 秒');
    expect(formatUptime(3_725)).toBe('1 小时 2 分 5 秒');
    expect(formatUptime(90_061)).toBe('1 天 1 小时 1 分 1 秒');
  });

  it('never returns negative durations', () => {
    expect(formatUptime(-10)).toBe('0 秒');
  });
});

describe('formatDateTime', () => {
  it('returns a placeholder for missing or invalid values', () => {
    expect(formatDateTime(null)).toBe('—');
    expect(formatDateTime('not-a-date')).toBe('—');
  });

  it('formats a valid ISO timestamp', () => {
    expect(formatDateTime('2026-11-03T12:00:00Z')).not.toBe('—');
  });
});

describe('formatRelativeTime', () => {
  const now = new Date('2026-11-03T12:00:00Z');

  it('describes recent timestamps', () => {
    expect(formatRelativeTime('2026-11-03T11:59:30Z', now)).toBe('刚刚');
    expect(formatRelativeTime('2026-11-03T11:30:00Z', now)).toBe('30 分钟前');
    expect(formatRelativeTime('2026-11-03T09:00:00Z', now)).toBe('3 小时前');
    expect(formatRelativeTime('2026-11-01T12:00:00Z', now)).toBe('2 天前');
  });
});

describe('formatLatency', () => {
  it('formats milliseconds and missing values', () => {
    expect(formatLatency(12)).toBe('12 ms');
    expect(formatLatency(undefined)).toBe('—');
  });
});
