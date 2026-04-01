import { create } from "zustand";

import type { PomodoroSnapshot, PomodoroTransition } from "../types/pomodoro";
import { defaultPomodoroSnapshot } from "../types/pomodoro";

interface PomodoroStore {
  hydrated: boolean;
  snapshot: PomodoroSnapshot;
  transition: PomodoroTransition | null;
  setSnapshot: (snapshot: PomodoroSnapshot) => void;
  setTransition: (transition: PomodoroTransition | null) => void;
}

export const usePomodoroStore = create<PomodoroStore>((set) => ({
  hydrated: false,
  snapshot: defaultPomodoroSnapshot,
  transition: null,
  setSnapshot: (snapshot) =>
    set({
      hydrated: true,
      snapshot,
    }),
  setTransition: (transition) => set({ transition }),
}));
