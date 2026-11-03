import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { dependencyLabel, toneFromStatus } from '@/lib/status';

import { StatusBadge } from './status-badge';

describe('StatusBadge', () => {
  it('renders the default label for a tone', () => {
    render(<StatusBadge tone="up" />);
    expect(screen.getByText('正常')).toBeInTheDocument();
  });

  it('renders a custom label', () => {
    render(<StatusBadge tone="down" label="连接失败" />);
    expect(screen.getByText('连接失败')).toBeInTheDocument();
  });
});

describe('toneFromStatus', () => {
  it('maps API status strings to visual tones', () => {
    expect(toneFromStatus('up')).toBe('up');
    expect(toneFromStatus('healthy')).toBe('up');
    expect(toneFromStatus('ready')).toBe('up');
    expect(toneFromStatus('down')).toBe('down');
    expect(toneFromStatus('degraded')).toBe('down');
    expect(toneFromStatus('not_ready')).toBe('down');
    expect(toneFromStatus(undefined)).toBe('unknown');
  });
});

describe('dependencyLabel', () => {
  it('maps dependency identifiers to display names', () => {
    expect(dependencyLabel('postgresql')).toBe('PostgreSQL');
    expect(dependencyLabel('redis')).toBe('Redis');
    expect(dependencyLabel('unknown')).toBe('unknown');
  });
});
