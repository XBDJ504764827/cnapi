import { useUiStore, type Theme } from '@/stores/ui-store';

const themeLabels: Record<Theme, string> = {
  light: '浅色',
  dark: '深色',
  system: '跟随系统',
};

/** Reads and updates the theme preference. */
export function useTheme() {
  const theme = useUiStore((state) => state.theme);
  const setTheme = useUiStore((state) => state.setTheme);

  return { theme, setTheme, themeLabel: themeLabels[theme] };
}

export { themeLabels };
