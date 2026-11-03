import { Bell } from 'lucide-react';

import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { Button } from '@/components/ui/button';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';
import { EmptyState } from '@/components/common/empty-state';

/** Notification entry point. The notification service does not exist yet. */
export function NotificationsMenu() {
  return (
    <Popover>
      <Tooltip>
        <TooltipTrigger asChild>
          <PopoverTrigger asChild>
            <Button variant="ghost" size="icon-sm" aria-label="通知">
              <Bell />
            </Button>
          </PopoverTrigger>
        </TooltipTrigger>
        <TooltipContent>通知</TooltipContent>
      </Tooltip>
      <PopoverContent className="w-80">
        <div className="flex items-center justify-between border-b border-border px-3.5 py-2.5">
          <p className="text-[13px] font-medium">通知</p>
          <span className="text-[11px] text-muted-foreground">未接入</span>
        </div>
        <EmptyState
          compact
          title="暂无通知"
          description="通知中心尚未接入，暂不展示任何消息。"
          className="m-2 border-0 bg-transparent"
        />
      </PopoverContent>
    </Popover>
  );
}
