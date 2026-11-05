import { LogIn } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/public/components/ui/button';
import { Callout } from '@/public/components/ui/callout';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/public/components/ui/dialog';

export function SteamLogin() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Button variant="outline" size="sm" onClick={() => setOpen(true)} className="hidden sm:inline-flex">
        <LogIn />
        Steam 登录
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>Steam 登录尚未启用</DialogTitle>
            <DialogDescription>
              公开门户当前无需登录即可浏览全部公开数据。Steam OpenID 登录与个人资料绑定将在后续阶段接入。
            </DialogDescription>
          </DialogHeader>
          <Callout
            variant="info"
            title="公开数据无需登录"
            description="玩家、地图、成绩、排行榜与服务器信息均为公开只读数据。"
            className="mt-4"
          />
          <DialogFooter>
            <DialogClose asChild>
              <Button size="sm" variant="outline">
                知道了
              </Button>
            </DialogClose>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  );
}
