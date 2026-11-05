import { describe, expect, it } from 'vitest';

import { formatNumber, formatPercent, formatSeconds, timeTypeLabel, tierLabel } from './format';

describe('public formatting helpers', () => {
  it('normalizes SQL decimal seconds without float parsing', () => {
    expect(formatSeconds('12.5')).toBe('12.500');
    expect(formatSeconds('12.345678901')).toBe('12.345');
    expect(formatSeconds('0.0078125')).toBe('0.007');
    expect(formatSeconds(null)).toBe('—');
  });

  it('formats percentages and numbers', () => {
    expect(formatPercent(1)).toBe('100.0%');
    expect(formatPercent(0.3333)).toBe('33.3%');
    expect(formatPercent(null)).toBe('—');
    expect(formatNumber(1234)).toBe('1,234');
    expect(formatNumber(undefined)).toBe('—');
  });

  it('labels time types and tiers', () => {
    expect(timeTypeLabel('pro')).toBe('PRO');
    expect(timeTypeLabel('nub')).toBe('NUB');
    expect(timeTypeLabel(null)).toBe('—');
    expect(tierLabel(3)).toBe('T3');
    expect(tierLabel(null)).toBe('未评级');
  });
});
