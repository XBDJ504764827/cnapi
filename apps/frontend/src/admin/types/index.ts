import type { LucideIcon } from 'lucide-react';

/** Shared UI level types. */

export type StatusTone = 'up' | 'down' | 'unknown';

export interface NavItem {
  label: string;
  to: string;
  icon: LucideIcon;
  /** Short description used by the command menu and page tooltips. */
  description?: string;
  /** Extra search keywords (Chinese + English). */
  keywords?: string[];
  /** Shown as a small badge, e.g. `规划中`. */
  badge?: string;
}

export interface NavSection {
  id: string;
  label: string;
  items: NavItem[];
}

export type SortDirection = 'asc' | 'desc';
