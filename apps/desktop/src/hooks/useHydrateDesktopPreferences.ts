import { useQuery } from "@tanstack/react-query";
import { useEffect } from "react";

import { getUserPreferences } from "../lib/storage";
import { usePomodoroPreferencesStore } from "../stores/pomodoro-preferences-store";
import { useThemeStore } from "../stores/theme-store";

export function useHydrateDesktopPreferences() {
  const hydrateThemeMode = useThemeStore((state) => state.hydrateMode);
  const setSoundEnabled = usePomodoroPreferencesStore(
    (state) => state.setSoundEnabled,
  );
  const preferencesQuery = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
    staleTime: 60_000,
  });

  useEffect(() => {
    if (!preferencesQuery.data) {
      return;
    }

    hydrateThemeMode(preferencesQuery.data.theme);
    setSoundEnabled(preferencesQuery.data.soundEnabled);
  }, [
    hydrateThemeMode,
    preferencesQuery.data,
    preferencesQuery.data?.soundEnabled,
    preferencesQuery.data?.theme,
    setSoundEnabled,
  ]);
}
