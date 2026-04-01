import { useQuery } from "@tanstack/react-query";
import { useEffect, useEffectEvent } from "react";

import { desktopListen } from "../lib/desktop-api";
import {
  getPomodoroState,
  POMODORO_STATE_EVENT,
  POMODORO_TRANSITION_EVENT,
} from "../lib/pomodoro";
import { usePomodoroStore } from "../stores/pomodoro-store";
import type { PomodoroEvent, PomodoroTransition } from "../types/pomodoro";

export function usePomodoroBridge() {
  const setSnapshot = usePomodoroStore((state) => state.setSnapshot);
  const setTransition = usePomodoroStore((state) => state.setTransition);

  const syncSnapshot = useEffectEvent((payload: PomodoroEvent) => {
    setSnapshot(payload.state);
  });

  const syncTransition = useEffectEvent((transition: PomodoroTransition) => {
    setTransition(transition);
    setSnapshot(transition.state);
  });

  const stateQuery = useQuery({
    queryKey: ["pomodoro-state"],
    queryFn: getPomodoroState,
    staleTime: Infinity,
    retry: false,
  });

  useEffect(() => {
    if (stateQuery.data) {
      setSnapshot(stateQuery.data);
    }
  }, [setSnapshot, stateQuery.data]);

  useEffect(() => {
    let active = true;
    let unlistenState: (() => void) | undefined;
    let unlistenTransition: (() => void) | undefined;

    async function attach() {
      try {
        unlistenState = await desktopListen<PomodoroEvent>(
          POMODORO_STATE_EVENT,
          (event) => {
            if (active) {
              syncSnapshot(event.payload);
            }
          },
        );
        unlistenTransition = await desktopListen<PomodoroTransition>(
          POMODORO_TRANSITION_EVENT,
          (event) => {
            if (active) {
              syncTransition(event.payload);
            }
          },
        );
      } catch {
        // The web-only preview does not provide the Tauri event bridge.
      }
    }

    void attach();

    return () => {
      active = false;
      void unlistenState?.();
      void unlistenTransition?.();
    };
  }, [syncSnapshot, syncTransition]);
}
