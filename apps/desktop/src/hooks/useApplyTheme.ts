import { useEffect, useEffectEvent } from "react";

import { useThemeStore } from "../stores/theme-store";

export function useApplyTheme() {
  const mode = useThemeStore((state) => state.mode);
  const setResolvedTheme = useThemeStore((state) => state.setResolvedTheme);

  const applyTheme = useEffectEvent((matchesDark: boolean) => {
    const resolvedTheme =
      mode === "system" ? (matchesDark ? "dark" : "light") : mode;

    document.documentElement.dataset.theme = resolvedTheme;
    setResolvedTheme(resolvedTheme);
  });

  useEffect(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleChange = () => applyTheme(mediaQuery.matches);

    handleChange();
    mediaQuery.addEventListener("change", handleChange);

    return () => mediaQuery.removeEventListener("change", handleChange);
  }, [applyTheme, mode]);
}
