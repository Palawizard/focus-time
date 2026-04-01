export type SessionStatus =
  | "planned"
  | "in_progress"
  | "completed"
  | "cancelled";
export type SessionSegmentKind = "focus" | "break" | "idle";
export type ThemePreference = "system" | "light" | "dark";
export type StatsPeriod = "day" | "week" | "month";
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

export interface StatsRange {
  startDate: string;
  endDate: string;
  comparisonStartDate: string;
  comparisonEndDate: string;
  isPartial: boolean;
}

export interface StatsSummary {
  focusSeconds: number;
  breakSeconds: number;
  totalSessions: number;
  completedSessions: number;
  interruptedSessions: number;
  activeDays: number;
  completionRate: number;
  averageFocusSecondsPerActiveDay: number;
  streakDays: number;
  bestStreakDays: number;
}

export interface StatsComparison {
  focusSecondsDelta: number;
  focusSecondsRatio: number | null;
  completionRateDelta: number;
  completedSessionsDelta: number;
  interruptedSessionsDelta: number;
  activeDaysDelta: number;
}

export interface StatsSeriesBucket {
  key: string;
  label: string;
  shortLabel: string;
  focusSeconds: number;
  breakSeconds: number;
  completedSessions: number;
  interruptedSessions: number;
}

export interface StatsWeekdayBucket {
  weekday: string;
  label: string;
  focusSeconds: number;
  shareRatio: number;
}

export interface StatsAppDistribution {
  trackedAppId: number | null;
  name: string;
  executable: string | null;
  category: TrackingCategory | null;
  colorHex: string | null;
  focusSeconds: number;
}

export interface StatsDashboard {
  period: StatsPeriod;
  range: StatsRange;
  summary: StatsSummary;
  comparison: StatsComparison;
  topApp: StatsAppDistribution | null;
  appDistribution: StatsAppDistribution[];
  focusSeries: StatsSeriesBucket[];
  weekdayDistribution: StatsWeekdayBucket[];
  isEmpty: boolean;
}

export interface Streak {
  currentDays: number;
  bestDays: number;
  todayCompleted: boolean;
  lastActiveDate: string | null;
  nextMilestoneDays: number;
  isAtRisk: boolean;
}

export interface WeeklyGoalProgress {
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
}

export interface ProgressBadge {
  slug: string;
  title: string;
  description: string;
  progressLabel: string;
  progressRatio: number;
  isUnlocked: boolean;
}

export interface AchievementProgress {
  slug: string;
  title: string;
  description: string;
  progressCurrent: number;
  progressTarget: number;
  progressRatio: number;
  unlockedAt: string | null;
}

export interface GamificationOverview {
  streak: Streak;
  weeklyGoal: WeeklyGoalProgress;
  badges: ProgressBadge[];
  achievements: AchievementProgress[];
}

export interface BackupArchiveSummary {
  fileName: string;
  path: string;
  createdAt: string;
  sizeBytes: number;
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
  soundEnabled: boolean;
  weeklyFocusGoalMinutes: number;
  weeklyCompletedSessionsGoal: number;
  launchOnStartup: boolean;
  trayEnabled: boolean;
  closeToTray: boolean;
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
