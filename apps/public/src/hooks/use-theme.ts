import { usePublicUiStore } from '@/stores/ui-store';

export function useTheme() {
  const theme = usePublicUiStore((state) => state.theme);
  const setTheme = usePublicUiStore((state) => state.setTheme);
  return { theme, setTheme };
}
