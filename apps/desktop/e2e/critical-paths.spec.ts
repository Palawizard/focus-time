import { expect, test, type Page } from "@playwright/test";

type FocusTimeE2EBridge = {
  invoke: (
    command: string,
    args?: Record<string, unknown>,
  ) => Promise<unknown>;
  listen: (
    eventName: string,
    handler: (event: { payload: unknown }) => void,
  ) => Promise<() => void>;
};

test.beforeEach(async ({ page }) => {
  await installDesktopBridge(page);
});

test("saves settings and updates desktop behavior", async ({ page }) => {
  await page.goto("/#/settings");

  await page.getByLabel("Focus minutes").fill("40");
  await page.getByRole("combobox").selectOption("dark");
  await page.getByLabel("Launch on startup").check();
  await page.getByLabel("Show tray icon").uncheck();
  await page.getByRole("button", { name: "Save settings" }).click();

  await expect(page.getByText("Settings saved.")).toBeVisible();
  await expect(page.getByText("40/5/15 min")).toBeVisible();
  await expect(page.getByText("Autostart plugin")).toBeVisible();
  await expect(page.getByText("Enabled", { exact: true })).toBeVisible();
  await expect(page.getByText("Hidden", { exact: true })).toBeVisible();
});

test("creates and restores a local backup", async ({ page }) => {
  await page.goto("/#/settings");

  await page.getByRole("button", { name: "Create backup" }).click();
  await expect(page.getByText(/Backup created at/)).toBeVisible();
  await expect(
    page.getByText("focus-time-backup-1.json", { exact: true }),
  ).toBeVisible();

  await page.getByLabel("Focus minutes").fill("55");
  await page.getByRole("button", { name: "Save settings" }).click();
  await expect(page.getByText("Settings saved.")).toBeVisible();

  await page
    .locator("div")
    .filter({ hasText: "focus-time-backup-1.json" })
    .getByRole("button", { name: "Restore" })
    .click();

  await expect(page.getByText(/Backup restored from/)).toBeVisible();
  await expect(page.getByLabel("Focus minutes")).toHaveValue("25");
});

test("runs a basic focus session journey", async ({ page }) => {
  await page.goto("/#/");

  await page.getByRole("button", { name: "Start" }).click();
  await expect(page.getByRole("button", { name: "Pause" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();

  await page.getByRole("button", { name: "Stop" }).click();
  await expect(page.getByRole("button", { name: "Start" })).toBeVisible();
  await expect(page.getByText("Ready", { exact: true })).toBeVisible();
});

async function installDesktopBridge(page: Page) {
  await page.addInitScript(() => {
    type Preferences = ReturnType<typeof buildPreferences>;
    type PomodoroState = {
      controlState: "idle" | "running" | "paused";
      phase: "focus" | "shortBreak" | "longBreak" | null;
      preset: {
        label: string;
        focusMinutes: number;
        shortBreakMinutes: number;
        longBreakMinutes: number;
        sessionsUntilLongBreak: number;
        autoStartBreaks: boolean;
        autoStartFocus: boolean;
      };
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
      outcome: string | null;
    };
    type RuntimeHealth = {
      productName: string;
      appVersion: string;
      desktopShell: string;
      platform: string;
      persistenceMode: string;
      workspaceCrates: string[];
      appDataDir: string;
      appLogDir: string;
      backupDir: string;
      launchOnStartupEnabled: boolean;
      trayEnabled: boolean;
      closeToTray: boolean;
    };
    type RuntimeLog = {
      path: string;
      contents: string;
    };
    type BackupEntry = {
      fileName: string;
      path: string;
      createdAt: string;
      sizeBytes: number;
      snapshot: {
        preferences: Preferences;
        runtimeHealth: RuntimeHealth;
      };
    };
    type BridgeState = {
      preferences: Preferences;
      runtimeHealth: RuntimeHealth;
      runtimeLog: RuntimeLog;
      backups: BackupEntry[];
      sessions: Array<Record<string, unknown>>;
      gamification: {
        streak: {
          currentDays: number;
          bestDays: number;
          todayCompleted: boolean;
          lastActiveDate: string | null;
          nextMilestoneDays: number;
          isAtRisk: boolean;
        };
        weeklyGoal: {
          startDate: string;
          endDate: string;
          focusGoalMinutes: number;
          completedSessionsGoal: number;
          focusMinutesCompleted: number;
          completedSessions: number;
          focusCompletionRatio: number;
          sessionsCompletionRatio: number;
          completedGoalCount: number;
          isCompleted: boolean;
        };
        badges: unknown[];
        achievements: unknown[];
      };
      trackingStatus: {
        status: {
          mode: string;
          capability: string;
          message: string;
          dependencyHint: string | null;
        };
        trackingEnabled: boolean;
        permissionGranted: boolean;
        onboardingCompleted: boolean;
        activeSessionId: number | null;
        activeWindow: unknown;
        lastError: string | null;
        isTrackingLive: boolean;
      };
      pomodoro: PomodoroState;
    };

    const listeners = new Map<
      string,
      Array<(event: { payload: unknown }) => void>
    >();
    let backupCounter = 0;
    let transitionId = 0;

    const clone = <T>(value: T): T => JSON.parse(JSON.stringify(value));
    const now = () => new Date().toISOString();
    const buildPreferences = () => ({
      focusMinutes: 25,
      shortBreakMinutes: 5,
      longBreakMinutes: 15,
      sessionsUntilLongBreak: 4,
      autoStartBreaks: false,
      autoStartFocus: false,
      trackingEnabled: true,
      trackingPermissionGranted: true,
      trackingOnboardingCompleted: true,
      notificationsEnabled: true,
      soundEnabled: false,
      weeklyFocusGoalMinutes: 240,
      weeklyCompletedSessionsGoal: 5,
      launchOnStartup: false,
      trayEnabled: true,
      closeToTray: true,
      theme: "system",
      updatedAt: now(),
    });
    const buildPomodoroState = (): PomodoroState => ({
      controlState: "idle",
      phase: null,
      preset: {
        label: "Custom",
        focusMinutes: 25,
        shortBreakMinutes: 5,
        longBreakMinutes: 15,
        sessionsUntilLongBreak: 4,
        autoStartBreaks: false,
        autoStartFocus: false,
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
    });

    const state: BridgeState = {
      preferences: buildPreferences(),
      runtimeHealth: {
        productName: "Focus Time",
        appVersion: "0.1.0",
        desktopShell: "Tauri v2",
        platform: "linux-x86_64",
        persistenceMode: "sqlite",
        workspaceCrates: [
          "focus-domain",
          "focus-persistence",
          "focus-stats",
          "focus-tracking",
        ],
        appDataDir: "/tmp/focus-time/data",
        appLogDir: "/tmp/focus-time/logs",
        backupDir: "/tmp/focus-time/backups",
        launchOnStartupEnabled: false,
        trayEnabled: true,
        closeToTray: true,
      },
      runtimeLog: {
        path: "/tmp/focus-time/logs/focus-time.log",
        contents: "[INFO] Focus Time desktop runtime ready",
      },
      backups: [],
      sessions: [],
      gamification: {
        streak: {
          currentDays: 0,
          bestDays: 0,
          todayCompleted: false,
          lastActiveDate: null,
          nextMilestoneDays: 3,
          isAtRisk: false,
        },
        weeklyGoal: {
          startDate: "2026-03-30",
          endDate: "2026-04-05",
          focusGoalMinutes: 240,
          completedSessionsGoal: 5,
          focusMinutesCompleted: 0,
          completedSessions: 0,
          focusCompletionRatio: 0,
          sessionsCompletionRatio: 0,
          completedGoalCount: 0,
          isCompleted: false,
        },
        badges: [],
        achievements: [],
      },
      trackingStatus: {
        status: {
          mode: "linux_wayland",
          capability: "supported",
          message: "Tracking ready.",
          dependencyHint: null,
        },
        trackingEnabled: true,
        permissionGranted: true,
        onboardingCompleted: true,
        activeSessionId: null,
        activeWindow: null,
        lastError: null,
        isTrackingLive: false,
      },
      pomodoro: buildPomodoroState(),
    };

    const emit = (eventName: string, payload: unknown) => {
      const handlers = listeners.get(eventName) ?? [];
      handlers.forEach((handler) => handler({ payload }));
    };

    const syncRuntime = () => {
      state.runtimeHealth.launchOnStartupEnabled =
        state.preferences.launchOnStartup;
      state.runtimeHealth.trayEnabled = state.preferences.trayEnabled;
      state.runtimeHealth.closeToTray = state.preferences.closeToTray;
    };

    const emitPomodoroState = () => {
      emit("pomodoro://state", { state: clone(state.pomodoro) });
    };

    const emitPomodoroTransition = (
      kind: string,
      title: string,
      body: string,
    ) => {
      transitionId += 1;
      emit("pomodoro://transition", {
        id: transitionId,
        kind,
        title,
        body,
        state: clone(state.pomodoro),
      });
    };

    const bridgeWindow = window as Window & {
      __FOCUS_TIME_E2E_API__?: FocusTimeE2EBridge;
    };

    bridgeWindow.__FOCUS_TIME_E2E_API__ = {
      invoke: async (command: string, args?: Record<string, unknown>) => {
        switch (command) {
          case "get_user_preferences":
            return clone(state.preferences);
          case "save_user_preferences": {
            state.preferences = {
              ...state.preferences,
              ...(args?.request as Record<string, unknown>),
              updatedAt: now(),
            };
            syncRuntime();
            return clone(state.preferences);
          }
          case "get_runtime_health":
            return clone(state.runtimeHealth);
          case "read_recent_runtime_log":
            return clone(state.runtimeLog);
          case "list_local_backups":
            return state.backups.map(({ snapshot, ...backup }: BackupEntry) => {
              void snapshot;
              return clone(backup);
            });
          case "create_local_backup": {
            backupCounter += 1;
            const backup = {
              fileName: `focus-time-backup-${backupCounter}.json`,
              path: `/tmp/focus-time/backups/focus-time-backup-${backupCounter}.json`,
              createdAt: now(),
              sizeBytes: 2048,
              snapshot: {
                preferences: clone(state.preferences),
                runtimeHealth: clone(state.runtimeHealth),
              },
            };

            state.backups.unshift(backup);

            return clone({
              fileName: backup.fileName,
              path: backup.path,
              createdAt: backup.createdAt,
              sizeBytes: backup.sizeBytes,
            });
          }
          case "restore_local_backup": {
            const path = (args?.request as { path: string }).path;
            const backup = state.backups.find(
              (entry: BackupEntry) => entry.path === path,
            );

            if (!backup) {
              throw new Error(`Unknown backup ${path}`);
            }

            state.preferences = {
              ...clone(backup.snapshot.preferences),
              updatedAt: now(),
            };
            state.runtimeHealth = clone(backup.snapshot.runtimeHealth);

            return clone({
              fileName: backup.fileName,
              path: backup.path,
              createdAt: backup.createdAt,
              sizeBytes: backup.sizeBytes,
            });
          }
          case "get_pomodoro_state":
            return clone(state.pomodoro);
          case "start_pomodoro": {
            const request = (args?.request as Record<string, unknown>) ?? {};

            state.pomodoro = {
              ...clone(state.pomodoro),
              controlState: "running",
              phase: "focus",
              preset: {
                label: String(request.label ?? "Custom"),
                focusMinutes: Number(request.focusMinutes ?? 25),
                shortBreakMinutes: Number(request.shortBreakMinutes ?? 5),
                longBreakMinutes: Number(request.longBreakMinutes ?? 15),
                sessionsUntilLongBreak: Number(
                  request.sessionsUntilLongBreak ?? 4,
                ),
                autoStartBreaks: Boolean(request.autoStartBreaks),
                autoStartFocus: Boolean(request.autoStartFocus),
              },
              remainingSeconds: Number(request.focusMinutes ?? 25) * 60,
              phaseTotalSeconds: Number(request.focusMinutes ?? 25) * 60,
              canPause: true,
              canResume: false,
              canStop: true,
              canSkipBreak: false,
              sessionId: 1,
            };
            emitPomodoroState();
            emitPomodoroTransition(
              "nextFocusStarted",
              "Focus started",
              "The next focus block is now running.",
            );
            return clone(state.pomodoro);
          }
          case "pause_pomodoro":
            state.pomodoro = {
              ...clone(state.pomodoro),
              controlState: "paused",
              canPause: false,
              canResume: true,
            };
            emitPomodoroState();
            return clone(state.pomodoro);
          case "resume_pomodoro":
            state.pomodoro = {
              ...clone(state.pomodoro),
              controlState: "running",
              canPause: true,
              canResume: false,
            };
            emitPomodoroState();
            return clone(state.pomodoro);
          case "stop_pomodoro":
          case "skip_pomodoro_break":
            state.pomodoro = buildPomodoroState();
            emitPomodoroState();
            return clone(state.pomodoro);
          case "list_sessions":
            return clone(state.sessions);
          case "get_gamification_overview":
            return clone(state.gamification);
          case "get_tracking_status":
            return clone(state.trackingStatus);
          case "list_tracked_apps":
          case "list_tracked_window_events":
          case "list_tracking_exclusion_rules":
          case "list_daily_stats":
            return [];
          default:
            throw new Error(`Unhandled desktop command: ${command}`);
        }
      },
      listen: async (
        eventName: string,
        handler: (event: { payload: unknown }) => void,
      ) => {
        const handlers = listeners.get(eventName) ?? [];
        handlers.push(handler);
        listeners.set(eventName, handlers);

        return () => {
          const nextHandlers = (listeners.get(eventName) ?? []).filter(
            (candidate) => candidate !== handler,
          );
          listeners.set(eventName, nextHandlers);
        };
      },
    };
  });
}
