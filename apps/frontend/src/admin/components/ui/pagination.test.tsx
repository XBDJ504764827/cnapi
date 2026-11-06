import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Pagination } from './pagination';

describe('Pagination', () => {
  it('renders the summary and navigates to a page', () => {
    const onPageChange = vi.fn();
    render(<Pagination page={1} pageSize={10} total={95} onPageChange={onPageChange} />);

    expect(screen.getByText(/共 95 条/)).toBeInTheDocument();
    expect(screen.getByText(/第 1 \/ 10 页/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: '第 2 页' }));
    expect(onPageChange).toHaveBeenCalledWith(2);

    fireEvent.click(screen.getByRole('button', { name: '下一页' }));
    expect(onPageChange).toHaveBeenCalledWith(2);
  });

  it('disables previous on the first page and next on the last page', () => {
    const { rerender } = render(<Pagination page={1} pageSize={10} total={10} onPageChange={() => {}} />);
    expect(screen.getByRole('button', { name: '上一页' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '下一页' })).toBeDisabled();

    rerender(<Pagination page={2} pageSize={10} total={25} onPageChange={() => {}} />);
    expect(screen.getByRole('button', { name: '上一页' })).toBeEnabled();
    expect(screen.getByRole('button', { name: '下一页' })).toBeEnabled();
  });

  it('keeps pagination disabled without data', () => {
    render(<Pagination page={1} pageSize={20} total={0} onPageChange={() => {}} />);
    expect(screen.getByText(/共 0 条/)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: '上一页' })).toBeDisabled();
    expect(screen.getByRole('button', { name: '下一页' })).toBeDisabled();
  });
});
