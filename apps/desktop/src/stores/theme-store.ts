import { create } from "zustand";

export type ThemeMode = "dark" | "light" | "system";
export type ResolvedTheme = "dark" | "light";

interface ThemeStore {
  mode: ThemeMode;
  resolvedTheme: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
  hydrateMode: (mode: ThemeMode) => void;
  setResolvedTheme: (theme: ResolvedTheme) => void;
}

const STORAGE_KEY = "focus-time-theme-mode";

function getStoredThemeMode(): ThemeMode {
  if (typeof window === "undefined") {
    return "system";
  }

  const storedMode = window.localStorage.getItem(STORAGE_KEY);

  return storedMode === "dark" ||
    storedMode === "light" ||
    storedMode === "system"
    ? storedMode
    : "system";
}

export const useThemeStore = create<ThemeStore>((set) => ({
  mode: getStoredThemeMode(),
  resolvedTheme: "dark",
  setMode: (mode) => {
    if (typeof window !== "undefined") {
      window.localStorage.setItem(STORAGE_KEY, mode);
    }

    set({ mode });
  },
  hydrateMode: (mode) => set({ mode }),
  setResolvedTheme: (resolvedTheme) => set({ resolvedTheme }),
}));
