import { create } from "zustand";

interface PomodoroPreferencesStore {
  soundEnabled: boolean;
  toggleSound: () => void;
}

const STORAGE_KEY = "focus-time-pomodoro-sound";

function getStoredSoundEnabled() {
  if (typeof window === "undefined") {
    return false;
  }

  return window.localStorage.getItem(STORAGE_KEY) === "true";
}

export const usePomodoroPreferencesStore = create<PomodoroPreferencesStore>(
  (set) => ({
    soundEnabled: getStoredSoundEnabled(),
    toggleSound: () =>
      set((state) => {
        const soundEnabled = !state.soundEnabled;

        if (typeof window !== "undefined") {
          window.localStorage.setItem(STORAGE_KEY, String(soundEnabled));
        }

        return { soundEnabled };
      }),
  }),
);
