export type SessionStatus = "planned" | "in_progress" | "completed" | "cancelled";
export type SessionSegmentKind = "focus" | "break" | "idle";
export type ThemePreference = "system" | "light" | "dark";

export interface Session {
  id: number;
  startedAt: string;
  endedAt: string | null;
  plannedFocusMinutes: number;
  actualFocusSeconds: number;
  breakSeconds: number;
  status: SessionStatus;
  presetLabel: string | null;
  note: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SessionSegment {
  id: number;
  sessionId: number;
  trackedAppId: number | null;
  kind: SessionSegmentKind;
  windowTitle: string | null;
  startedAt: string;
  endedAt: string;
  durationSeconds: number;
  createdAt: string;
}

export interface TrackedApp {
  id: number;
  name: string;
  executable: string;
  colorHex: string | null;
  isExcluded: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface DailyStat {
  date: string;
  focusSeconds: number;
  breakSeconds: number;
  completedSessions: number;
  interruptedSessions: number;
  topAppId: number | null;
  updatedAt: string;
}

export interface UserPreference {
  focusMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsUntilLongBreak: number;
  autoStartBreaks: boolean;
  autoStartFocus: boolean;
  trackingEnabled: boolean;
  notificationsEnabled: boolean;
  theme: ThemePreference;
  updatedAt: string;
}

export interface CreateSessionRequest {
  plannedFocusMinutes: number;
  status: SessionStatus;
  presetLabel?: string | null;
  note?: string | null;
}

export interface CreateSessionSegmentRequest {
  sessionId: number;
  trackedAppId?: number | null;
  kind: SessionSegmentKind;
  windowTitle?: string | null;
  startedAt: string;
  endedAt: string;
  durationSeconds: number;
}

export interface UpsertTrackedAppRequest {
  name: string;
  executable: string;
  colorHex?: string | null;
  isExcluded: boolean;
}

export interface SaveDailyStatRequest {
  date: string;
  focusSeconds: number;
  breakSeconds: number;
  completedSessions: number;
  interruptedSessions: number;
  topAppId?: number | null;
}

export interface DevelopmentSeedReport {
  skipped: boolean;
  sessionsInserted: number;
  trackedAppsUpserted: number;
  dailyStatsUpserted: number;
}
