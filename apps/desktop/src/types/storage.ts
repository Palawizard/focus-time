export type SessionStatus = "planned" | "in_progress" | "completed" | "cancelled";
export type SessionSegmentKind = "focus" | "break" | "idle";
export type ThemePreference = "system" | "light" | "dark";
export type TrackingCategory =
  | "development"
  | "browser"
  | "communication"
  | "writing"
  | "design"
  | "meeting"
  | "research"
  | "utilities"
  | "unknown";
export type TrackingExclusionKind = "executable" | "window_title" | "category";
export type TrackingMode =
  | "windows_native"
  | "linux_hyprland"
  | "linux_sway"
  | "linux_x11"
  | "linux_wayland"
  | "unsupported";
export type TrackingCapability = "supported" | "limited" | "unsupported";

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
  category: TrackingCategory;
  colorHex: string | null;
  isExcluded: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface TrackedWindowEvent {
  id: number;
  sessionId: number | null;
  trackedAppId: number | null;
  appName: string | null;
  executable: string | null;
  category: TrackingCategory | null;
  windowTitle: string | null;
  startedAt: string;
  endedAt: string | null;
  createdAt: string;
}

export interface TrackingExclusionRule {
  id: number;
  kind: TrackingExclusionKind;
  pattern: string;
  createdAt: string;
  updatedAt: string;
}

export interface TrackingStatus {
  mode: TrackingMode;
  capability: TrackingCapability;
  message: string;
  dependencyHint: string | null;
}

export interface ActiveWindowSample {
  appName: string;
  executable: string;
  category: TrackingCategory;
  windowTitle: string | null;
}

export interface TrackingRuntimeSnapshot {
  status: TrackingStatus;
  trackingEnabled: boolean;
  permissionGranted: boolean;
  onboardingCompleted: boolean;
  activeSessionId: number | null;
  activeWindow: ActiveWindowSample | null;
  lastError: string | null;
  isTrackingLive: boolean;
}

export interface SessionHistoryFilters {
  dateFrom?: string | null;
  dateTo?: string | null;
  minDurationSeconds?: number | null;
  maxDurationSeconds?: number | null;
  presetLabel?: string | null;
  status?: SessionStatus | null;
  trackedAppId?: number | null;
}

export interface HistorySessionApp {
  trackedAppId: number;
  name: string;
  executable: string;
  category: TrackingCategory;
  colorHex: string | null;
  durationSeconds: number;
}

export interface HistorySessionSummary {
  session: Session;
  totalDurationSeconds: number;
  trackedApps: HistorySessionApp[];
  interruptionCount: number;
  interruptionSeconds: number;
}

export interface HistorySessionsPage {
  items: HistorySessionSummary[];
  nextOffset: number | null;
}

export interface SessionSegmentDetail {
  segment: SessionSegment;
  trackedApp: TrackedApp | null;
}

export interface HistorySessionDetail {
  session: Session;
  totalDurationSeconds: number;
  trackedApps: HistorySessionApp[];
  segments: SessionSegmentDetail[];
  trackedWindowEvents: TrackedWindowEvent[];
  interruptionCount: number;
  interruptionSeconds: number;
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
  trackingPermissionGranted: boolean;
  trackingOnboardingCompleted: boolean;
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
  category: TrackingCategory;
  colorHex?: string | null;
  isExcluded: boolean;
}

export interface CreateTrackingExclusionRuleRequest {
  kind: TrackingExclusionKind;
  pattern: string;
}

export interface ReplaceSessionRequest {
  sessionId: number;
  startedAt: string;
  endedAt?: string | null;
  plannedFocusMinutes: number;
  actualFocusSeconds: number;
  breakSeconds: number;
  status: SessionStatus;
  presetLabel?: string | null;
  note?: string | null;
}

export type HistoryExportFormat = "csv" | "json";

export interface ExportHistoryRequest {
  format: HistoryExportFormat;
  filters?: SessionHistoryFilters | null;
}

export interface ExportHistoryResult {
  path: string;
  format: HistoryExportFormat;
  sessionsExported: number;
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
