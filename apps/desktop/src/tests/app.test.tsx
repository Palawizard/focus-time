import { render, screen } from "@testing-library/react";

import { App } from "../app/App";

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => undefined),
}));

vi.mock("../lib/tauri", () => ({
  getRuntimeHealth: vi.fn().mockResolvedValue({
    productName: "Focus Time",
    desktopShell: "Tauri v2",
    platform: "windows-x86_64",
    persistenceMode: "sqlite-planned",
    workspaceCrates: [
      "focus-domain",
      "focus-persistence",
      "focus-stats",
      "focus-tracking",
    ],
  }),
}));

vi.mock("../lib/pomodoro", () => ({
  getPomodoroState: vi.fn().mockResolvedValue({
    controlState: "idle",
    phase: null,
    preset: {
      label: "Classic",
      focusMinutes: 25,
      shortBreakMinutes: 5,
      longBreakMinutes: 15,
      sessionsUntilLongBreak: 4,
    },
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
  }),
  pausePomodoro: vi.fn(),
  resumePomodoro: vi.fn(),
  skipPomodoroBreak: vi.fn(),
  startPomodoro: vi.fn(),
  stopPomodoro: vi.fn(),
  POMODORO_STATE_EVENT: "pomodoro://state",
  POMODORO_TRANSITION_EVENT: "pomodoro://transition",
}));

vi.mock("../lib/storage", async () => {
  const actual = await vi.importActual("../lib/storage");

  return {
    ...actual,
    getUserPreferences: vi.fn().mockResolvedValue({
      focusMinutes: 25,
      shortBreakMinutes: 5,
      longBreakMinutes: 15,
      sessionsUntilLongBreak: 4,
      autoStartBreaks: false,
      autoStartFocus: false,
      trackingEnabled: true,
      notificationsEnabled: true,
      theme: "system",
      updatedAt: "2026-03-29T00:00:00Z",
    }),
    listSessions: vi.fn().mockResolvedValue([]),
  };
});

describe("App", () => {
  it("renders the Focus Time shell", async () => {
    render(<App />);

    expect(screen.getByText("Focus Time")).toBeInTheDocument();
    expect(screen.getByText("Overview")).toBeInTheDocument();
  });
});
