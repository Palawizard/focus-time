use chrono::NaiveDate;
use focus_domain::{DailyStat, Session, SessionSegment, SessionSegmentKind, SessionStatus, TrackedApp, UserPreference};
use focus_persistence::UpsertTrackedAppInput;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    pub planned_focus_minutes: i32,
    pub status: String,
    pub preset_label: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionSegmentRequest {
    pub session_id: i64,
    pub tracked_app_id: Option<i64>,
    pub kind: String,
    pub window_title: Option<String>,
    pub started_at: String,
    pub ended_at: String,
    pub duration_seconds: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveUserPreferencesRequest {
    pub focus_minutes: i32,
    pub short_break_minutes: i32,
    pub long_break_minutes: i32,
    pub sessions_until_long_break: i32,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
    pub tracking_enabled: bool,
    pub notifications_enabled: bool,
    pub theme: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTrackedAppRequest {
    pub name: String,
    pub executable: String,
    pub color_hex: Option<String>,
    pub is_excluded: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDailyStatRequest {
    pub date: String,
    pub focus_seconds: i64,
    pub break_seconds: i64,
    pub completed_sessions: i32,
    pub interrupted_sessions: i32,
    pub top_app_id: Option<i64>,
}

#[tauri::command]
pub async fn list_sessions(
    state: tauri::State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<Session>, String> {
    state
        .storage
        .list_sessions(limit.unwrap_or(30))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_session(
    state: tauri::State<'_, AppState>,
    request: CreateSessionRequest,
) -> Result<Session, String> {
    state
        .storage
        .create_session(
            request.planned_focus_minutes,
            parse_session_status(&request.status)?,
            request.preset_label,
            request.note,
        )
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_session_segments(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<Vec<SessionSegment>, String> {
    state
        .storage
        .list_session_segments(session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_session_segment(
    state: tauri::State<'_, AppState>,
    request: CreateSessionSegmentRequest,
) -> Result<SessionSegment, String> {
    state
        .storage
        .create_session_segment(focus_persistence::CreateSessionSegmentInput {
            session_id: request.session_id,
            tracked_app_id: request.tracked_app_id,
            kind: parse_segment_kind(&request.kind)?,
            window_title: request.window_title,
            started_at: parse_utc_timestamp(&request.started_at)?,
            ended_at: parse_utc_timestamp(&request.ended_at)?,
            duration_seconds: request.duration_seconds,
        })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_user_preferences(
    state: tauri::State<'_, AppState>,
) -> Result<UserPreference, String> {
    state
        .storage
        .get_user_preferences()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_user_preferences(
    state: tauri::State<'_, AppState>,
    request: SaveUserPreferencesRequest,
) -> Result<UserPreference, String> {
    let current = state
        .storage
        .get_user_preferences()
        .await
        .map_err(|error| error.to_string())?;

    let preferences = UserPreference {
        focus_minutes: request.focus_minutes,
        short_break_minutes: request.short_break_minutes,
        long_break_minutes: request.long_break_minutes,
        sessions_until_long_break: request.sessions_until_long_break,
        auto_start_breaks: request.auto_start_breaks,
        auto_start_focus: request.auto_start_focus,
        tracking_enabled: request.tracking_enabled,
        notifications_enabled: request.notifications_enabled,
        theme: parse_theme(&request.theme)?,
        updated_at: current.updated_at,
    };

    state
        .storage
        .save_user_preferences(&preferences)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_tracked_apps(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TrackedApp>, String> {
    state
        .storage
        .list_tracked_apps()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn upsert_tracked_app(
    state: tauri::State<'_, AppState>,
    request: UpsertTrackedAppRequest,
) -> Result<TrackedApp, String> {
    state
        .storage
        .upsert_tracked_app(UpsertTrackedAppInput {
            name: request.name,
            executable: request.executable,
            color_hex: request.color_hex,
            is_excluded: request.is_excluded,
        })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_daily_stats(
    state: tauri::State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<DailyStat>, String> {
    state
        .storage
        .list_daily_stats(limit.unwrap_or(30))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_daily_stat(
    state: tauri::State<'_, AppState>,
    request: SaveDailyStatRequest,
) -> Result<DailyStat, String> {
    let date =
        NaiveDate::parse_from_str(&request.date, "%Y-%m-%d").map_err(|error| error.to_string())?;

    state
        .storage
        .upsert_daily_stat(
            date,
            request.focus_seconds,
            request.break_seconds,
            request.completed_sessions,
            request.interrupted_sessions,
            request.top_app_id,
        )
        .await
        .map_err(|error| error.to_string())
}

fn parse_session_status(value: &str) -> Result<SessionStatus, String> {
    match value {
        "planned" => Ok(SessionStatus::Planned),
        "in_progress" => Ok(SessionStatus::InProgress),
        "completed" => Ok(SessionStatus::Completed),
        "cancelled" => Ok(SessionStatus::Cancelled),
        _ => Err(format!("unsupported session status: {value}")),
    }
}

fn parse_segment_kind(value: &str) -> Result<SessionSegmentKind, String> {
    match value {
        "focus" => Ok(SessionSegmentKind::Focus),
        "break" => Ok(SessionSegmentKind::Break),
        "idle" => Ok(SessionSegmentKind::Idle),
        _ => Err(format!("unsupported segment kind: {value}")),
    }
}

fn parse_theme(value: &str) -> Result<focus_domain::ThemePreference, String> {
    match value {
        "system" => Ok(focus_domain::ThemePreference::System),
        "light" => Ok(focus_domain::ThemePreference::Light),
        "dark" => Ok(focus_domain::ThemePreference::Dark),
        _ => Err(format!("unsupported theme: {value}")),
    }
}

fn parse_utc_timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
        .map_err(|error| error.to_string())
}
