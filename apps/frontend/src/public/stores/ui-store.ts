import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type Theme = 'light' | 'dark' | 'system';

interface PublicUiState {
  theme: Theme;
  setTheme: (theme: Theme) => void;
}

/** Public portal preferences (separate localStorage key from the admin app). */
export const usePublicUiStore = create<PublicUiState>()(
  persist(
    (set) => ({
      theme: 'dark',
      setTheme: (theme) => set({ theme }),
    }),
    { name: 'cngokz-public-ui' },
  ),
);
