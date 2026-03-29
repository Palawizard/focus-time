import { useQuery } from "@tanstack/react-query";
import { useEffect, useRef } from "react";

import { getUserPreferences } from "../lib/storage";
import { usePomodoroPreferencesStore } from "../stores/pomodoro-preferences-store";
import { usePomodoroStore } from "../stores/pomodoro-store";
import type { PomodoroTransitionKind } from "../types/pomodoro";

const notifyKinds = new Set<PomodoroTransitionKind>([
  "focusCompleted",
  "breakCompleted",
  "nextFocusStarted",
  "breakStarted",
]);

export function usePomodoroNotifications() {
  const transition = usePomodoroStore((state) => state.transition);
  const soundEnabled = usePomodoroPreferencesStore((state) => state.soundEnabled);
  const lastHandledTransitionId = useRef<number | null>(null);

  const userPreferences = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
    staleTime: 60_000,
  });

  useEffect(() => {
    if (!transition || lastHandledTransitionId.current === transition.id) {
      return;
    }

    lastHandledTransitionId.current = transition.id;

    if (!notifyKinds.has(transition.kind)) {
      return;
    }

    if (userPreferences.data?.notificationsEnabled ?? true) {
      void showDesktopNotification(transition.title, transition.body);
    }

    if (soundEnabled) {
      playNotificationTone();
    }
  }, [soundEnabled, transition, userPreferences.data?.notificationsEnabled]);
}

async function showDesktopNotification(title: string, body: string) {
  if (typeof window === "undefined" || !("Notification" in window)) {
    return;
  }

  if (window.Notification.permission === "granted") {
    new window.Notification(title, { body });
    return;
  }

  if (window.Notification.permission !== "denied") {
    const permission = await window.Notification.requestPermission();

    if (permission === "granted") {
      new window.Notification(title, { body });
    }
  }
}

function playNotificationTone() {
  if (typeof window === "undefined") {
    return;
  }

  const AudioContextClass =
    window.AudioContext ||
    (window as Window & { webkitAudioContext?: typeof AudioContext })
      .webkitAudioContext;

  if (!AudioContextClass) {
    return;
  }

  const context = new AudioContextClass();
  const oscillator = context.createOscillator();
  const gainNode = context.createGain();

  oscillator.type = "sine";
  oscillator.frequency.value = 880;
  gainNode.gain.value = 0.03;

  oscillator.connect(gainNode);
  gainNode.connect(context.destination);

  oscillator.start();
  oscillator.stop(context.currentTime + 0.16);
  void context.close();
}
