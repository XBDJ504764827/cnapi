import type { LucideIcon } from 'lucide-react';

/** Shared UI level types. */

export type StatusTone = 'up' | 'down' | 'unknown';

export interface NavItem {
  label: string;
  to: string;
  icon: LucideIcon;
  /** Shown as a small badge, e.g. `预留` for planned modules. */
  badge?: string;
}

export interface NavSection {
  label: string;
  items: NavItem[];
}
