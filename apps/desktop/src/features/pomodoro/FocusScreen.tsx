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

  const preferencePreset: StartPomodoroRequest = {
    label: "Custom",
    focusMinutes: userPreferences.data?.focusMinutes ?? 25,
    shortBreakMinutes: userPreferences.data?.shortBreakMinutes ?? 5,
    longBreakMinutes: userPreferences.data?.longBreakMinutes ?? 15,
    sessionsUntilLongBreak: userPreferences.data?.sessionsUntilLongBreak ?? 4,
    autoStartBreaks: userPreferences.data?.autoStartBreaks ?? false,
    autoStartFocus: userPreferences.data?.autoStartFocus ?? false,
  };
  const phaseLabel =
    snapshot.phase === "focus"
      ? "Focus"
      : snapshot.phase === "shortBreak"
        ? "Short break"
        : snapshot.phase === "longBreak"
          ? "Long break"
          : "Ready";
  const statusLabel =
    snapshot.controlState === "running"
      ? "Running"
      : snapshot.controlState === "paused"
        ? "Paused"
        : snapshot.outcome === "completed"
          ? "Complete"
          : snapshot.outcome === "interrupted"
            ? "Stopped"
            : snapshot.outcome === "skippedBreak"
              ? "Break skipped"
              : "Idle";
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
            <CardTitle className="mt-3">{snapshot.preset.label}</CardTitle>
          </Card>
          <Card>
            <CardDescription>Phase</CardDescription>
            <CardTitle className="mt-3">{phaseLabel}</CardTitle>
          </Card>
          <Card>
            <CardDescription>Status</CardDescription>
            <CardTitle className="mt-3">{statusLabel}</CardTitle>
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
            <Button onClick={() => startMutation.mutate(preferencePreset)}>
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
            <button
              key={preset.label}
              className="ft-panel-muted rounded-[1rem] px-4 py-4 text-left transition-colors hover:bg-[var(--color-brand-soft)]"
              onClick={() =>
                startMutation.mutate({
                  ...preset,
                  autoStartBreaks: preferencePreset.autoStartBreaks,
                  autoStartFocus: preferencePreset.autoStartFocus,
                })
              }
              type="button"
            >
              <p className="text-sm font-medium">{preset.label}</p>
              <p className="ft-text-muted mt-2 text-sm">
                {preset.focusMinutes} min focus · {preset.shortBreakMinutes} min
                break
              </p>
            </button>
          ))}
        </div>
      </Card>

      <Card className="ft-panel p-6">
        <CardHeader>
          <CardDescription>Current session</CardDescription>
          <CardTitle>
            {snapshot.controlState === "idle"
              ? "Ready for the next block."
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
              {snapshot.autoStartBreaks ? "On" : "Off"}
            </p>
          </div>
          <div className="ft-panel-muted px-4 py-3">
            <p className="ft-text-muted text-sm">Auto-start focus</p>
            <p className="mt-2 text-sm">
              {snapshot.autoStartFocus ? "On" : "Off"}
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

function formatDuration(totalSeconds: number) {
  const minutes = Math.floor(totalSeconds / 60)
    .toString()
    .padStart(2, "0");
  const seconds = Math.floor(totalSeconds % 60)
    .toString()
    .padStart(2, "0");

  return `${minutes}:${seconds}`;
}
