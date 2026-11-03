import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { TierBadge } from './tier-badge';

describe('TierBadge', () => {
  it('renders a tier label', () => {
    render(<TierBadge tier={4} />);
    expect(screen.getByText('T4')).toBeInTheDocument();
  });

  it('renders an unrated placeholder', () => {
    render(<TierBadge tier={null} />);
    expect(screen.getByText('—')).toBeInTheDocument();
  });
});
