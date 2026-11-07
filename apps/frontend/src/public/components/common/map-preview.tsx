import { cn } from '@/public/lib/utils';

/**
 * 地图封面：优先 Steam CDN 的 workshop 预览图，加载失败/无图时退化为首字母块。
 * onError 里挂上占位，避免图片挂了留下破图标。
 */
export function MapPreview({
  name,
  url,
  className,
  rounded = 'md',
}: {
  name: string;
  url?: string | null;
  className?: string;
  rounded?: 'md' | 'lg' | 'full';
}) {
  const radius = rounded === 'full' ? 'rounded-full' : rounded === 'lg' ? 'rounded-lg' : 'rounded-md';

  if (!url) {
    return (
      <span
        className={cn(
          'flex shrink-0 items-center justify-center border border-border bg-surface-elevated text-[10px] font-semibold text-muted-foreground',
          radius,
          className,
        )}
        aria-hidden="true"
      >
        {name.slice(0, 2).toUpperCase()}
      </span>
    );
  }

  return (
    <img
      src={url}
      alt={`${name} 预览图`}
      loading="lazy"
      decoding="async"
      className={cn('shrink-0 border border-border object-cover', radius, className)}
      onError={(event) => {
        event.currentTarget.style.visibility = 'hidden';
      }}
    />
  );
}
