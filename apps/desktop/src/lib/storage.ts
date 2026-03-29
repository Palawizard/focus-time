import { invoke } from "@tauri-apps/api/core";

import type {
  CreateSessionRequest,
  CreateSessionSegmentRequest,
  DailyStat,
  SaveDailyStatRequest,
  Session,
  SessionSegment,
  TrackedApp,
  UpsertTrackedAppRequest,
  UserPreference,
} from "../types/storage";

export function listSessions(limit = 30) {
  return invoke<Session[]>("list_sessions", { limit });
}

export function createSession(request: CreateSessionRequest) {
  return invoke<Session>("create_session", { request });
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

export function saveUserPreferences(request: Omit<UserPreference, "updatedAt">) {
  return invoke<UserPreference>("save_user_preferences", { request });
}

export function listTrackedApps() {
  return invoke<TrackedApp[]>("list_tracked_apps");
}

export function upsertTrackedApp(request: UpsertTrackedAppRequest) {
  return invoke<TrackedApp>("upsert_tracked_app", { request });
}

export function listDailyStats(limit = 30) {
  return invoke<DailyStat[]>("list_daily_stats", { limit });
}

export function saveDailyStat(request: SaveDailyStatRequest) {
  return invoke<DailyStat>("save_daily_stat", { request });
}
