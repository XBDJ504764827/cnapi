import { ArrowRight, Search } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router';

import { Dialog, DialogContent, DialogTitle } from '@/admin/components/ui/dialog';
import { Kbd } from '@/admin/components/ui/kbd';
import { cn } from '@/admin/lib/utils';

import { searchNavItems } from './nav-config';

interface CommandMenuProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** Global page search (⌘K / Ctrl+K). Only navigates to real, existing pages. */
export function CommandMenu({ open, onOpenChange }: CommandMenuProps) {
  const navigate = useNavigate();
  const [query, setQuery] = useState('');
  const [activeIndex, setActiveIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  const results = useMemo(() => searchNavItems(query), [query]);

  const handleOpenChange = (nextOpen: boolean) => {
    if (!nextOpen) {
      setQuery('');
      setActiveIndex(0);
    }
    onOpenChange(nextOpen);
  };

  const select = (to: string) => {
    onOpenChange(false);
    navigate(to);
  };

  const handleKeyDown = (event: React.KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      setActiveIndex((index) => Math.min(index + 1, Math.max(results.length - 1, 0)));
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      setActiveIndex((index) => Math.max(index - 1, 0));
    } else if (event.key === 'Enter') {
      event.preventDefault();
      const selected = results[activeIndex];
      if (selected) select(selected.to);
    }
  };

  useEffect(() => {
    const element = listRef.current?.querySelector<HTMLElement>(`[data-index="${activeIndex}"]`);
    element?.scrollIntoView({ block: 'nearest' });
  }, [activeIndex]);

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent
        className="top-[15%] max-w-xl translate-y-0 gap-0 overflow-hidden p-0"
        aria-describedby={undefined}
      >
        <DialogTitle className="sr-only">搜索页面</DialogTitle>
        <div className="flex items-center gap-2 border-b border-border px-3.5">
          <Search className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          <input
            autoFocus
            value={query}
            onChange={(event) => {
              setQuery(event.target.value);
              setActiveIndex(0);
            }}
            onKeyDown={handleKeyDown}
            placeholder="搜索页面、功能或关键词…"
            className="h-12 w-full bg-transparent text-[13px] outline-none placeholder:text-muted-foreground/70"
            aria-label="搜索页面"
          />
          <Kbd>ESC</Kbd>
        </div>

        <div ref={listRef} className="max-h-[min(60vh,420px)] overflow-y-auto p-1.5">
          {results.length === 0 ? (
            <div className="flex flex-col items-center gap-1 px-4 py-10 text-center">
              <p className="text-[13px] font-medium">没有匹配的页面</p>
              <p className="text-xs text-muted-foreground">试试“玩家”“成绩”“API 文档”等关键词。</p>
            </div>
          ) : (
            <ul role="listbox" aria-label="页面搜索结果">
              {results.map((item, index) => {
                const Icon = item.icon;
                return (
                  <li key={item.to}>
                    <button
                      type="button"
                      role="option"
                      aria-selected={index === activeIndex}
                      data-index={index}
                      onMouseEnter={() => setActiveIndex(index)}
                      onClick={() => select(item.to)}
                      className={cn(
                        'flex w-full items-center gap-2.5 rounded-md px-2.5 py-2 text-left text-[13px] transition-colors',
                        index === activeIndex ? 'bg-surface-hover text-foreground' : 'text-muted-foreground',
                      )}
                    >
                      <Icon className="size-4 shrink-0" aria-hidden="true" />
                      <span className="min-w-0 flex-1">
                        <span className="block truncate text-foreground">{item.label}</span>
                        {item.description ? (
                          <span className="block truncate text-[11px] text-muted-foreground">{item.description}</span>
                        ) : null}
                      </span>
                      <span className="hidden shrink-0 text-[10px] text-muted-foreground sm:block">
                        {item.sectionLabel}
                      </span>
                      <ArrowRight className="size-3.5 shrink-0 opacity-40" aria-hidden="true" />
                    </button>
                  </li>
                );
              })}
            </ul>
          )}
        </div>

        <div className="flex items-center justify-between border-t border-border bg-surface-elevated/40 px-3.5 py-2 text-[11px] text-muted-foreground">
          <span className="flex items-center gap-1.5">
            <Kbd>↑</Kbd>
            <Kbd>↓</Kbd>
            导航
            <Kbd>↵</Kbd>
            打开
          </span>
          <span>共 {results.length} 个页面</span>
        </div>
      </DialogContent>
    </Dialog>
  );
}
