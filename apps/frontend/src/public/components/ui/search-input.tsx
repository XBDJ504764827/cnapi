import { Search, X } from 'lucide-react';
import type * as React from 'react';

import { Input } from '@/public/components/ui/input';
import { Kbd } from '@/public/components/ui/kbd';
import { cn } from '@/public/lib/utils';

interface SearchInputProps extends Omit<React.ComponentProps<'input'>, 'onChange' | 'value'> {
  value: string;
  onChange: (value: string) => void;
  shortcut?: string;
}

/** Search field with icon, clear button and optional shortcut hint. */
export function SearchInput({
  value,
  onChange,
  className,
  placeholder = '搜索…',
  shortcut,
  ...props
}: SearchInputProps) {
  return (
    <div className={cn('relative', className)}>
      <Search
        className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground"
        aria-hidden="true"
      />
      <Input
        type="search"
        value={value}
        placeholder={placeholder}
        onChange={(event) => onChange(event.target.value)}
        className={cn(
          'pl-8',
          value ? 'pr-16' : shortcut ? 'pr-16' : 'pr-8',
          '[&::-webkit-search-cancel-button]:appearance-none',
        )}
        {...props}
      />
      {value ? (
        <button
          type="button"
          onClick={() => onChange('')}
          className="absolute top-1/2 right-2 flex size-5 -translate-y-1/2 items-center justify-center rounded-sm text-muted-foreground hover:bg-surface-hover hover:text-foreground"
          aria-label="清空搜索"
        >
          <X className="size-3.5" />
        </button>
      ) : shortcut ? (
        <span className="pointer-events-none absolute top-1/2 right-2 -translate-y-1/2">
          <Kbd>{shortcut}</Kbd>
        </span>
      ) : null}
    </div>
  );
}
