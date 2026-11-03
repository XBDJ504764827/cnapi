import { Check, Copy } from 'lucide-react';
import { useState } from 'react';

import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

interface CodeBlockProps {
  code: string;
  label?: string;
  className?: string;
}

/** Read-only code sample with a real copy action. */
export function CodeBlock({ code, label, className }: CodeBlockProps) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1500);
    } catch {
      setCopied(false);
    }
  };

  return (
    <div className={cn('relative overflow-hidden rounded-md border border-border bg-surface-elevated/50', className)}>
      {label ? (
        <p className="border-b border-border/70 px-3 py-1.5 font-mono text-[10px] text-muted-foreground">{label}</p>
      ) : null}
      <pre className="overflow-x-auto px-3 py-2.5 pr-12 font-mono text-[11px] leading-relaxed text-foreground/90">
        <code>{code}</code>
      </pre>
      <Button
        variant="ghost"
        size="icon-xs"
        className="absolute top-1.5 right-1.5"
        onClick={() => void copy()}
        aria-label="复制代码"
      >
        {copied ? <Check className="text-success" /> : <Copy />}
      </Button>
    </div>
  );
}
