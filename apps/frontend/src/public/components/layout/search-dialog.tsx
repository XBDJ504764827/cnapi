import { Search } from 'lucide-react';
import { useState } from 'react';

import { SearchResults } from '@/public/components/common/search-results';
import { Dialog, DialogContent, DialogTitle } from '@/public/components/ui/dialog';
import { Kbd } from '@/public/components/ui/kbd';
import { Input } from '@/public/components/ui/input';

export function SearchDialog({ open, onOpenChange }: { open: boolean; onOpenChange: (open: boolean) => void }) {
  const [query, setQuery] = useState('');

  const handleOpenChange = (nextOpen: boolean) => {
    if (!nextOpen) setQuery('');
    onOpenChange(nextOpen);
  };

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="top-[12%] max-w-xl translate-y-0 gap-0 p-0" aria-describedby={undefined}>
        <DialogTitle className="sr-only">搜索玩家或地图</DialogTitle>
        <div className="flex items-center gap-2 border-b border-border px-3.5">
          <Search className="size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          <Input
            autoFocus
            value={query}
            onChange={(event) => setQuery(event.target.value)}
            placeholder="搜索玩家昵称、SteamID64 或地图…"
            className="h-12 border-0 bg-transparent px-0 shadow-none focus-visible:ring-0"
            aria-label="搜索玩家或地图"
          />
          <Kbd>ESC</Kbd>
        </div>
        <div className="max-h-[60vh] overflow-y-auto p-2">
          <SearchResults query={query} onNavigate={() => handleOpenChange(false)} />
        </div>
      </DialogContent>
    </Dialog>
  );
}
