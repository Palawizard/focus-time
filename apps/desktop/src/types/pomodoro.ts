export type PomodoroPhase = "focus" | "shortBreak" | "longBreak";
export type PomodoroControlState = "idle" | "running" | "paused";
export type PomodoroSessionOutcome = "completed" | "interrupted" | "skippedBreak";
export type PomodoroTransitionKind =
  | "sessionStarted"
  | "paused"
  | "resumed"
  | "focusCompleted"
  | "breakStarted"
  | "breakCompleted"
  | "sessionStopped"
  | "breakSkipped"
  | "nextFocusStarted";

export interface PomodoroPreset {
  label: string;
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsUntilLongBreak: number;
}

export interface PomodoroSnapshot {
  controlState: PomodoroControlState;
  phase: PomodoroPhase | null;
  preset: PomodoroPreset;
  sessionStartedAt: string | null;
  phaseStartedAt: string | null;
  phaseEndsAt: string | null;
  pausedAt: string | null;
  remainingSeconds: number;
  phaseTotalSeconds: number;
  phaseElapsedSeconds: number;
  focusSecondsElapsed: number;
  breakSecondsElapsed: number;
  completedFocusBlocks: number;
  completedBreaks: number;
  autoStartBreaks: boolean;
  autoStartFocus: boolean;
  canPause: boolean;
  canResume: boolean;
  canStop: boolean;
  canSkipBreak: boolean;
  sessionId: number | null;
  outcome: PomodoroSessionOutcome | null;
}

export interface PomodoroTransition {
  id: number;
  kind: PomodoroTransitionKind;
  title: string;
  body: string;
  state: PomodoroSnapshot;
}

export interface PomodoroEvent {
  state: PomodoroSnapshot;
}

export interface StartPomodoroRequest {
  label: string;
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsUntilLongBreak: number;
  autoStartBreaks: boolean;
  autoStartFocus: boolean;
}

export const defaultPomodoroPreset: PomodoroPreset = {
  label: "Classic",
  focusMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  sessionsUntilLongBreak: 4,
};

export const defaultPomodoroSnapshot: PomodoroSnapshot = {
  controlState: "idle",
  phase: null,
  preset: defaultPomodoroPreset,
  sessionStartedAt: null,
  phaseStartedAt: null,
  phaseEndsAt: null,
  pausedAt: null,
  remainingSeconds: 0,
  phaseTotalSeconds: 0,
  phaseElapsedSeconds: 0,
  focusSecondsElapsed: 0,
  breakSecondsElapsed: 0,
  completedFocusBlocks: 0,
  completedBreaks: 0,
  autoStartBreaks: false,
  autoStartFocus: false,
  canPause: false,
  canResume: false,
  canStop: false,
  canSkipBreak: false,
  sessionId: null,
  outcome: null,
};
