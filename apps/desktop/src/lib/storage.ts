// noinspection JSUnusedGlobalSymbols
import type {
  BackupArchiveSummary,
  CreateTrackingExclusionRuleRequest,
  ExportHistoryRequest,
  ExportHistoryResult,
  GamificationOverview,
  CreateSessionRequest,
  CreateSessionSegmentRequest,
  DailyStat,
  DevelopmentSeedReport,
  HistorySessionDetail,
  HistorySessionsPage,
  ReplaceSessionRequest,
  SaveDailyStatRequest,
  Session,
  SessionSegment,
  SessionHistoryFilters,
  TrackedApp,
  TrackedWindowEvent,
  StatsDashboard,
  StatsPeriod,
  TrackingExclusionRule,
  TrackingRuntimeSnapshot,
  UpsertTrackedAppRequest,
  UserPreference,
} from "../types/storage";
import { desktopInvoke } from "./desktop-api";

export function listSessions(limit = 30) {
  return desktopInvoke<Session[]>("list_sessions", { limit });
}

export function listHistorySessions(
  limit = 20,
  offset = 0,
  filters?: SessionHistoryFilters,
) {
  return desktopInvoke<HistorySessionsPage>("list_history_sessions", {
    request: { limit, offset, filters },
  });
}

export function getHistorySessionDetail(sessionId: number) {
  return desktopInvoke<HistorySessionDetail>("get_history_session_detail", {
    sessionId,
  });
}

export function createSession(request: CreateSessionRequest) {
  return desktopInvoke<Session>("create_session", { request });
}

export function replaceSession(request: ReplaceSessionRequest) {
  return desktopInvoke<Session>("replace_session", { request });
}

export function deleteSession(sessionId: number) {
  return desktopInvoke<void>("delete_session", { sessionId });
}

export function listSessionSegments(sessionId: number) {
  return desktopInvoke<SessionSegment[]>("list_session_segments", {
    sessionId,
  });
}

export function createSessionSegment(request: CreateSessionSegmentRequest) {
  return desktopInvoke<SessionSegment>("create_session_segment", { request });
}

export function getUserPreferences() {
  return desktopInvoke<UserPreference>("get_user_preferences");
}

export function saveUserPreferences(
  request: Omit<UserPreference, "updatedAt">,
) {
  return desktopInvoke<UserPreference>("save_user_preferences", { request });
}

export function createLocalBackup() {
  return desktopInvoke<BackupArchiveSummary>("create_local_backup");
}

export function listLocalBackups() {
  return desktopInvoke<BackupArchiveSummary[]>("list_local_backups");
}

export function restoreLocalBackup(path: string) {
  return desktopInvoke<BackupArchiveSummary>("restore_local_backup", {
    request: { path },
  });
}

export function listTrackedApps() {
  return desktopInvoke<TrackedApp[]>("list_tracked_apps");
}

export function upsertTrackedApp(request: UpsertTrackedAppRequest) {
  return desktopInvoke<TrackedApp>("upsert_tracked_app", { request });
}

export function getTrackingStatus() {
  return desktopInvoke<TrackingRuntimeSnapshot>("get_tracking_status");
}

export function listTrackedWindowEvents(limit = 30) {
  return desktopInvoke<TrackedWindowEvent[]>("list_tracked_window_events", {
    limit,
  });
}

export function listTrackingExclusionRules() {
  return desktopInvoke<TrackingExclusionRule[]>(
    "list_tracking_exclusion_rules",
  );
}

export function createTrackingExclusionRule(
  request: CreateTrackingExclusionRuleRequest,
) {
  return desktopInvoke<TrackingExclusionRule>(
    "create_tracking_exclusion_rule",
    {
      request,
    },
  );
}

export function deleteTrackingExclusionRule(ruleId: number) {
  return desktopInvoke<void>("delete_tracking_exclusion_rule", { ruleId });
}

export function exportHistory(request: ExportHistoryRequest) {
  return desktopInvoke<ExportHistoryResult>("export_history", { request });
}

export function listDailyStats(limit = 30) {
  return desktopInvoke<DailyStat[]>("list_daily_stats", { limit });
}

export function getStatsDashboard(period: StatsPeriod) {
  return desktopInvoke<StatsDashboard>("get_stats_dashboard", { period });
}

export function getGamificationOverview() {
  return desktopInvoke<GamificationOverview>("get_gamification_overview");
}

export function saveDailyStat(request: SaveDailyStatRequest) {
  return desktopInvoke<DailyStat>("save_daily_stat", { request });
}

export function seedDevelopmentFixtures() {
  return desktopInvoke<DevelopmentSeedReport>("seed_development_fixtures");
}
