import { invoke } from "@tauri-apps/api/core";

import type {
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

export function listSessions(limit = 30) {
  return invoke<Session[]>("list_sessions", { limit });
}

export function listHistorySessions(
  limit = 20,
  offset = 0,
  filters?: SessionHistoryFilters,
) {
  return invoke<HistorySessionsPage>("list_history_sessions", {
    request: { limit, offset, filters },
  });
}

export function getHistorySessionDetail(sessionId: number) {
  return invoke<HistorySessionDetail>("get_history_session_detail", {
    sessionId,
  });
}

export function createSession(request: CreateSessionRequest) {
  return invoke<Session>("create_session", { request });
}

export function replaceSession(request: ReplaceSessionRequest) {
  return invoke<Session>("replace_session", { request });
}

export function deleteSession(sessionId: number) {
  return invoke<void>("delete_session", { sessionId });
}

export function listSessionSegments(sessionId: number) {
  return invoke<SessionSegment[]>("list_session_segments", { sessionId });
}

export function createSessionSegment(request: CreateSessionSegmentRequest) {
  return invoke<SessionSegment>("create_session_segment", { request });
}

export function getUserPreferences() {
  return invoke<UserPreference>("get_user_preferences");
}

export function saveUserPreferences(
  request: Omit<UserPreference, "updatedAt">,
) {
  return invoke<UserPreference>("save_user_preferences", { request });
}

export function listTrackedApps() {
  return invoke<TrackedApp[]>("list_tracked_apps");
}

export function upsertTrackedApp(request: UpsertTrackedAppRequest) {
  return invoke<TrackedApp>("upsert_tracked_app", { request });
}

export function getTrackingStatus() {
  return invoke<TrackingRuntimeSnapshot>("get_tracking_status");
}

export function listTrackedWindowEvents(limit = 30) {
  return invoke<TrackedWindowEvent[]>("list_tracked_window_events", { limit });
}

export function listTrackingExclusionRules() {
  return invoke<TrackingExclusionRule[]>("list_tracking_exclusion_rules");
}

export function createTrackingExclusionRule(
  request: CreateTrackingExclusionRuleRequest,
) {
  return invoke<TrackingExclusionRule>("create_tracking_exclusion_rule", {
    request,
  });
}

export function deleteTrackingExclusionRule(ruleId: number) {
  return invoke<void>("delete_tracking_exclusion_rule", { ruleId });
}

export function exportHistory(request: ExportHistoryRequest) {
  return invoke<ExportHistoryResult>("export_history", { request });
}

export function listDailyStats(limit = 30) {
  return invoke<DailyStat[]>("list_daily_stats", { limit });
}

export function getStatsDashboard(period: StatsPeriod) {
  return invoke<StatsDashboard>("get_stats_dashboard", { period });
}

export function getGamificationOverview() {
  return invoke<GamificationOverview>("get_gamification_overview");
}

export function saveDailyStat(request: SaveDailyStatRequest) {
  return invoke<DailyStat>("save_daily_stat", { request });
}

export function seedDevelopmentFixtures() {
  return invoke<DevelopmentSeedReport>("seed_development_fixtures");
}
