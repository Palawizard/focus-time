import { useEffect, useState } from "react";
import { useMutation, useQuery } from "@tanstack/react-query";

import { Button } from "../../components/ui/Button";
import { Card, CardDescription, CardHeader, CardTitle } from "../../components/ui/Card";
import {
  pausePomodoro,
  resumePomodoro,
  skipPomodoroBreak,
  startPomodoro,
  stopPomodoro,
} from "../../lib/pomodoro";
import { getUserPreferences, listSessions } from "../../lib/storage";
import { usePomodoroPreferencesStore } from "../../stores/pomodoro-preferences-store";
import { usePomodoroStore } from "../../stores/pomodoro-store";
import type { StartPomodoroRequest } from "../../types/pomodoro";

const quickPresets = [
  {
    label: "Classic",
    focusMinutes: 25,
    shortBreakMinutes: 5,
    longBreakMinutes: 15,
    sessionsUntilLongBreak: 4,
  },
  {
    label: "Deep Work",
    focusMinutes: 45,
    shortBreakMinutes: 10,
    longBreakMinutes: 20,
    sessionsUntilLongBreak: 3,
  },
  {
    label: "Sprint",
    focusMinutes: 60,
    shortBreakMinutes: 10,
    longBreakMinutes: 20,
    sessionsUntilLongBreak: 2,
  },
] as const;

export function FocusScreen() {
  const snapshot = usePomodoroStore((state) => state.snapshot);
  const soundEnabled = usePomodoroPreferencesStore((state) => state.soundEnabled);
  const toggleSound = usePomodoroPreferencesStore((state) => state.toggleSound);
  const userPreferences = useQuery({
    queryKey: ["user-preferences"],
    queryFn: getUserPreferences,
  });
  const recentSessions = useQuery({
    queryKey: ["recent-sessions"],
    queryFn: () => listSessions(3),
  });

  const startMutation = useMutation({
    mutationFn: (request: StartPomodoroRequest) => startPomodoro(request),
  });
  const pauseMutation = useMutation({ mutationFn: pausePomodoro });
  const resumeMutation = useMutation({ mutationFn: resumePomodoro });
  const stopMutation = useMutation({ mutationFn: stopPomodoro });
  const skipMutation = useMutation({ mutationFn: skipPomodoroBreak });

  const preferenceLabel = "Custom";
  const preferenceFocusMinutes = userPreferences.data?.focusMinutes ?? 25;
  const preferenceShortBreakMinutes =
    userPreferences.data?.shortBreakMinutes ?? 5;
  const preferenceLongBreakMinutes = userPreferences.data?.longBreakMinutes ?? 15;
  const preferenceSessionsUntilLongBreak =
    userPreferences.data?.sessionsUntilLongBreak ?? 4;
  const preferenceAutoStartBreaks =
    userPreferences.data?.autoStartBreaks ?? false;
  const preferenceAutoStartFocus = userPreferences.data?.autoStartFocus ?? false;
  const preferencePreset: StartPomodoroRequest = {
    label: preferenceLabel,
    focusMinutes: preferenceFocusMinutes,
    shortBreakMinutes: preferenceShortBreakMinutes,
    longBreakMinutes: preferenceLongBreakMinutes,
    sessionsUntilLongBreak: preferenceSessionsUntilLongBreak,
    autoStartBreaks: preferenceAutoStartBreaks,
    autoStartFocus: preferenceAutoStartFocus,
  };
  const [selectedPreset, setSelectedPreset] =
    useState<StartPomodoroRequest | null>(null);

  useEffect(() => {
    if (snapshot.controlState !== "idle" || selectedPreset !== null) {
      return;
    }

    setSelectedPreset({
      label: preferenceLabel,
      focusMinutes: preferenceFocusMinutes,
      shortBreakMinutes: preferenceShortBreakMinutes,
      longBreakMinutes: preferenceLongBreakMinutes,
      sessionsUntilLongBreak: preferenceSessionsUntilLongBreak,
      autoStartBreaks: preferenceAutoStartBreaks,
      autoStartFocus: preferenceAutoStartFocus,
    });
  }, [
    preferenceAutoStartBreaks,
    preferenceAutoStartFocus,
    preferenceFocusMinutes,
    preferenceLabel,
    preferenceLongBreakMinutes,
    preferenceSessionsUntilLongBreak,
    preferenceShortBreakMinutes,
    selectedPreset,
    snapshot.controlState,
  ]);

  const configuredPreset =
    snapshot.controlState === "idle"
      ? (selectedPreset ?? preferencePreset)
      : snapshot.preset;
  const phaseLabel =
    snapshot.phase === "focus"
      ? "Focus"
      : snapshot.phase === "shortBreak"
        ? "Short break"
        : snapshot.phase === "longBreak"
          ? "Long break"
          : "Ready";
  const remainingLabel = formatDuration(snapshot.remainingSeconds);
  const phaseProgress =
    snapshot.phaseTotalSeconds > 0
      ? Math.min(
          100,
          Math.round(
            (snapshot.phaseElapsedSeconds / snapshot.phaseTotalSeconds) * 100,
          ),
        )
      : 0;

  return (
    <div className="grid gap-5 xl:grid-cols-[minmax(0,1.35fr)_minmax(21rem,1fr)]">
      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Focus</CardDescription>
          <CardTitle>Run a full focus cycle with clear controls.</CardTitle>
        </CardHeader>

        <div className="mt-8 grid gap-4 md:grid-cols-3">
          <Card>
            <CardDescription>Preset</CardDescription>
            <CardTitle className="mt-3">{configuredPreset.label}</CardTitle>
          </Card>
          <Card>
            <CardDescription>Focus</CardDescription>
            <CardTitle className="mt-3">
              {configuredPreset.focusMinutes} min
            </CardTitle>
          </Card>
          <Card>
            <CardDescription>Break</CardDescription>
            <CardTitle className="mt-3">
              {configuredPreset.shortBreakMinutes} min
            </CardTitle>
          </Card>
        </div>

        <div className="ft-panel-strong mt-8 p-6 text-center">
          <p className="ft-kicker text-[11px] font-semibold">{phaseLabel}</p>
          <p className="ft-font-display mt-4 text-6xl font-semibold tracking-tight sm:text-7xl">
            {remainingLabel}
          </p>
          <div className="mx-auto mt-5 h-2 max-w-md overflow-hidden rounded-full bg-[var(--color-surface-muted)]">
            <div
              className="h-full rounded-full bg-[var(--color-brand)] transition-[width] duration-300"
              style={{ width: `${phaseProgress}%` }}
            />
          </div>
          <p className="ft-text-muted mt-4 text-sm">
            Focus {formatDuration(snapshot.focusSecondsElapsed)} · Break{" "}
            {formatDuration(snapshot.breakSecondsElapsed)}
          </p>
        </div>

        <div className="mt-8 flex flex-wrap gap-3">
          {snapshot.controlState === "idle" ? (
            <Button onClick={() => startMutation.mutate(configuredPreset)}>
              Start
            </Button>
          ) : null}
          {snapshot.canPause ? (
            <Button onClick={() => pauseMutation.mutate()}>Pause</Button>
          ) : null}
          {snapshot.canResume ? (
            <Button onClick={() => resumeMutation.mutate()}>Resume</Button>
          ) : null}
          {snapshot.canStop ? (
            <Button onClick={() => stopMutation.mutate()} variant="secondary">
              Stop
            </Button>
          ) : null}
          {snapshot.canSkipBreak ? (
            <Button onClick={() => skipMutation.mutate()} variant="ghost">
              Skip break
            </Button>
          ) : null}
          <Button onClick={toggleSound} variant="ghost">
            {soundEnabled ? "Sound on" : "Sound off"}
          </Button>
        </div>

        <div className="mt-8 grid gap-3 md:grid-cols-3">
          {quickPresets.map((preset) => (
            <PresetButton
              key={preset.label}
              active={
                snapshot.controlState === "idle" &&
                configuredPreset.label === preset.label
              }
              disabled={snapshot.controlState !== "idle"}
              onClick={() =>
                setSelectedPreset({
                  ...preset,
                  autoStartBreaks: preferenceAutoStartBreaks,
                  autoStartFocus: preferenceAutoStartFocus,
                })
              }
              preset={preset}
            />
          ))}
        </div>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Current session</CardDescription>
          <CardTitle>
            {snapshot.controlState === "idle"
              ? `${configuredPreset.label} is ready.`
              : `${snapshot.preset.label} in progress.`}
          </CardTitle>
        </CardHeader>

        <div className="mt-8 grid gap-3">
          <div className="ft-panel-muted px-4 py-3">
            <p className="ft-text-muted text-sm">Notifications</p>
            <p className="mt-2 text-sm">
              {userPreferences.data?.notificationsEnabled ? "On" : "Off"}
            </p>
          </div>
          <div className="ft-panel-muted px-4 py-3">
            <p className="ft-text-muted text-sm">Auto-start break</p>
            <p className="mt-2 text-sm">
              {configuredPreset.autoStartBreaks ? "On" : "Off"}
            </p>
          </div>
          <div className="ft-panel-muted px-4 py-3">
            <p className="ft-text-muted text-sm">Auto-start focus</p>
            <p className="mt-2 text-sm">
              {configuredPreset.autoStartFocus ? "On" : "Off"}
            </p>
          </div>
          <div className="ft-panel-muted px-4 py-3">
            <p className="ft-text-muted text-sm">Recent sessions</p>
            <div className="mt-3 space-y-2 text-sm">
              {recentSessions.data?.length ? (
                recentSessions.data.map((session) => (
                  <div
                    className="flex items-center justify-between"
                    key={session.id}
                  >
                    <span>{session.presetLabel ?? "Focus block"}</span>
                    <span className="ft-text-muted">
                      {session.plannedFocusMinutes} min
                    </span>
                  </div>
                ))
              ) : (
                <p className="ft-text-muted">No sessions yet.</p>
              )}
            </div>
          </div>
        </div>
      </Card>
    </div>
  );
}

interface PresetButtonProps {
  active: boolean;
  disabled: boolean;
  onClick: () => void;
  preset: (typeof quickPresets)[number];
}

function PresetButton({
  active,
  disabled,
  onClick,
  preset,
}: PresetButtonProps) {
  return (
    <button
      className={[
        "ft-panel-muted ft-interactive-panel rounded-[1rem] px-4 py-4 text-left",
        active
          ? "border-[var(--color-border-strong)] bg-[var(--color-brand-soft)] shadow-[0_14px_32px_rgba(0,0,0,0.12)]"
          : "",
        disabled ? "cursor-not-allowed opacity-60" : "",
      ].join(" ")}
      disabled={disabled}
      onClick={onClick}
      type="button"
    >
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-sm font-medium">{preset.label}</p>
          <p className="ft-text-muted mt-2 text-sm">
            {preset.focusMinutes} min focus · {preset.shortBreakMinutes} min
            break
          </p>
        </div>
        {active ? (
          <span className="ft-brand-badge rounded-full px-2.5 py-1 text-[11px] font-semibold">
            Ready
          </span>
        ) : null}
      </div>
    </button>
  );
}

function formatDuration(totalSeconds: number) {
  const minutes = Math.floor(totalSeconds / 60)
    .toString()
    .padStart(2, "0");
  const seconds = Math.floor(totalSeconds % 60)
    .toString()
    .padStart(2, "0");

  return `${minutes}:${seconds}`;
}
