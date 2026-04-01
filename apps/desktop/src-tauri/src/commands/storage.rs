use chrono::NaiveDate;
use focus_domain::{
    DailyStat, GamificationOverview, Session, SessionSegment, SessionSegmentKind, SessionStatus,
    TrackedApp, TrackedWindowEvent, TrackingCategory, TrackingExclusionKind, TrackingExclusionRule,
    UserPreference,
};
use focus_persistence::DevelopmentSeedReport;
use focus_persistence::UpsertTrackedAppInput;
use serde::Deserialize;
use tauri::Manager;

use crate::services::{
    BackupArchiveSummary, HistoryExportFormat, HistoryExportResult, HistoryFiltersInput,
    HistorySessionDetail, HistorySessionsPage, ReplaceSessionDetailsInput, StatsDashboard,
    StatsPeriod, TrackingRuntimeSnapshot,
};
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
    pub tracking_permission_granted: bool,
    pub tracking_onboarding_completed: bool,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub weekly_focus_goal_minutes: i32,
    pub weekly_completed_sessions_goal: i32,
    pub launch_on_startup: bool,
    pub tray_enabled: bool,
    pub close_to_tray: bool,
    pub theme: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpsertTrackedAppRequest {
    pub name: String,
    pub executable: String,
    pub category: String,
    pub color_hex: Option<String>,
    pub is_excluded: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTrackingExclusionRuleRequest {
    pub kind: String,
    pub pattern: String,
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

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SessionHistoryFiltersRequest {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub min_duration_seconds: Option<i64>,
    pub max_duration_seconds: Option<i64>,
    pub preset_label: Option<String>,
    pub status: Option<String>,
    pub tracked_app_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListHistorySessionsRequest {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub filters: Option<SessionHistoryFiltersRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceSessionRequest {
    pub session_id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub planned_focus_minutes: i32,
    pub actual_focus_seconds: i64,
    pub break_seconds: i64,
    pub status: String,
    pub preset_label: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportHistoryRequest {
    pub format: String,
    pub filters: Option<SessionHistoryFiltersRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreLocalBackupRequest {
    pub path: String,
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
pub async fn list_history_sessions(
    state: tauri::State<'_, AppState>,
    request: ListHistorySessionsRequest,
) -> Result<HistorySessionsPage, String> {
    state
        .storage
        .list_history_sessions(
            request.limit.unwrap_or(20),
            request.offset.unwrap_or_default(),
            parse_history_filters(request.filters)?,
        )
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_history_session_detail(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<HistorySessionDetail, String> {
    state
        .storage
        .get_history_session_detail(session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn replace_session(
    state: tauri::State<'_, AppState>,
    request: ReplaceSessionRequest,
) -> Result<Session, String> {
    state
        .storage
        .replace_session(ReplaceSessionDetailsInput {
            session_id: request.session_id,
            started_at: parse_utc_timestamp(&request.started_at)?,
            ended_at: request
                .ended_at
                .as_deref()
                .map(parse_utc_timestamp)
                .transpose()?,
            planned_focus_minutes: request.planned_focus_minutes,
            actual_focus_seconds: request.actual_focus_seconds,
            break_seconds: request.break_seconds,
            status: parse_session_status(&request.status)?,
            preset_label: request.preset_label,
            note: request.note,
        })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_session(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<(), String> {
    state
        .storage
        .delete_session(session_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_history(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: ExportHistoryRequest,
) -> Result<HistoryExportResult, String> {
    let export_root = app_handle
        .path()
        .download_dir()
        .or_else(|_| app_handle.path().document_dir())
        .unwrap_or_else(|_| {
            app_handle
                .path()
                .app_data_dir()
                .expect("app data dir should resolve")
        })
        .join("focus-time-exports");

    state
        .storage
        .export_history(
            export_root,
            parse_history_export_format(&request.format)?,
            parse_history_filters(request.filters)?,
        )
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
    app_handle: tauri::AppHandle,
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
        tracking_permission_granted: request.tracking_permission_granted,
        tracking_onboarding_completed: request.tracking_onboarding_completed,
        notifications_enabled: request.notifications_enabled,
        sound_enabled: request.sound_enabled,
        weekly_focus_goal_minutes: request.weekly_focus_goal_minutes,
        weekly_completed_sessions_goal: request.weekly_completed_sessions_goal,
        launch_on_startup: request.launch_on_startup,
        tray_enabled: request.tray_enabled,
        close_to_tray: request.close_to_tray,
        theme: parse_theme(&request.theme)?,
        updated_at: current.updated_at,
    };

    let saved_preferences = state
        .storage
        .save_user_preferences(&preferences)
        .await
        .map_err(|error| error.to_string())?;

    state
        .runtime
        .apply_user_preferences(&app_handle, &saved_preferences)
        .map_err(|error| error.to_string())?;

    Ok(saved_preferences)
}

#[tauri::command]
pub async fn create_local_backup(
    state: tauri::State<'_, AppState>,
) -> Result<BackupArchiveSummary, String> {
    state
        .storage
        .create_backup(state.runtime.backup_dir().to_path_buf())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_local_backups(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<BackupArchiveSummary>, String> {
    state
        .storage
        .list_backups(state.runtime.backup_dir().to_path_buf())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn restore_local_backup(
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: RestoreLocalBackupRequest,
) -> Result<BackupArchiveSummary, String> {
    let summary = state
        .storage
        .restore_backup(request.path.into())
        .await
        .map_err(|error| error.to_string())?;
    let preferences = state
        .storage
        .get_user_preferences()
        .await
        .map_err(|error| error.to_string())?;

    state
        .runtime
        .apply_user_preferences(&app_handle, &preferences)
        .map_err(|error| error.to_string())?;

    Ok(summary)
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
            category: parse_tracking_category(&request.category)?,
            color_hex: request.color_hex,
            is_excluded: request.is_excluded,
        })
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_tracking_status(
    state: tauri::State<'_, AppState>,
) -> Result<TrackingRuntimeSnapshot, String> {
    Ok(state.tracker.get_status().await)
}

#[tauri::command]
pub async fn list_tracked_window_events(
    state: tauri::State<'_, AppState>,
    limit: Option<u32>,
) -> Result<Vec<TrackedWindowEvent>, String> {
    state
        .tracker
        .list_recent_events(limit.unwrap_or(30))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_tracking_exclusion_rules(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<TrackingExclusionRule>, String> {
    state
        .tracker
        .list_exclusion_rules()
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_tracking_exclusion_rule(
    state: tauri::State<'_, AppState>,
    request: CreateTrackingExclusionRuleRequest,
) -> Result<TrackingExclusionRule, String> {
    state
        .tracker
        .create_exclusion_rule(
            parse_tracking_exclusion_kind(&request.kind)?,
            request.pattern,
        )
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_tracking_exclusion_rule(
    state: tauri::State<'_, AppState>,
    rule_id: i64,
) -> Result<(), String> {
    state
        .tracker
        .delete_exclusion_rule(rule_id)
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
pub async fn get_stats_dashboard(
    state: tauri::State<'_, AppState>,
    period: String,
) -> Result<StatsDashboard, String> {
    state
        .storage
        .get_stats_dashboard(parse_stats_period(&period)?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn get_gamification_overview(
    state: tauri::State<'_, AppState>,
) -> Result<GamificationOverview, String> {
    state
        .storage
        .get_gamification_overview()
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

#[tauri::command]
pub async fn seed_development_fixtures(
    state: tauri::State<'_, AppState>,
) -> Result<DevelopmentSeedReport, String> {
    state
        .storage
        .seed_development_data()
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

fn parse_tracking_category(value: &str) -> Result<TrackingCategory, String> {
    match value {
        "development" => Ok(TrackingCategory::Development),
        "browser" => Ok(TrackingCategory::Browser),
        "communication" => Ok(TrackingCategory::Communication),
        "writing" => Ok(TrackingCategory::Writing),
        "design" => Ok(TrackingCategory::Design),
        "meeting" => Ok(TrackingCategory::Meeting),
        "research" => Ok(TrackingCategory::Research),
        "utilities" => Ok(TrackingCategory::Utilities),
        "unknown" => Ok(TrackingCategory::Unknown),
        _ => Err(format!("unsupported tracking category: {value}")),
    }
}

fn parse_tracking_exclusion_kind(value: &str) -> Result<TrackingExclusionKind, String> {
    match value {
        "executable" => Ok(TrackingExclusionKind::Executable),
        "window_title" => Ok(TrackingExclusionKind::WindowTitle),
        "category" => Ok(TrackingExclusionKind::Category),
        _ => Err(format!("unsupported exclusion kind: {value}")),
    }
}

fn parse_stats_period(value: &str) -> Result<StatsPeriod, String> {
    match value {
        "day" => Ok(StatsPeriod::Day),
        "week" => Ok(StatsPeriod::Week),
        "month" => Ok(StatsPeriod::Month),
        _ => Err(format!("unsupported stats period: {value}")),
    }
}

fn parse_history_filters(
    filters: Option<SessionHistoryFiltersRequest>,
) -> Result<HistoryFiltersInput, String> {
    let filters = filters.unwrap_or_default();

    if let Some(date_from) = &filters.date_from {
        NaiveDate::parse_from_str(date_from, "%Y-%m-%d").map_err(|error| error.to_string())?;
    }

    if let Some(date_to) = &filters.date_to {
        NaiveDate::parse_from_str(date_to, "%Y-%m-%d").map_err(|error| error.to_string())?;
    }

    Ok(HistoryFiltersInput {
        date_from: filters.date_from,
        date_to: filters.date_to,
        min_duration_seconds: filters.min_duration_seconds,
        max_duration_seconds: filters.max_duration_seconds,
        preset_label: filters
            .preset_label
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        status: filters
            .status
            .as_deref()
            .map(parse_session_status)
            .transpose()?,
        tracked_app_id: filters.tracked_app_id,
    })
}

fn parse_history_export_format(value: &str) -> Result<HistoryExportFormat, String> {
    match value {
        "csv" => Ok(HistoryExportFormat::Csv),
        "json" => Ok(HistoryExportFormat::Json),
        _ => Err(format!("unsupported export format: {value}")),
    }
}

fn parse_utc_timestamp(value: &str) -> Result<chrono::DateTime<chrono::Utc>, String> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|timestamp| timestamp.with_timezone(&chrono::Utc))
        .map_err(|error| error.to_string())
}
