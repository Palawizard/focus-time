use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroPreset {
    pub label: String,
    pub focus_minutes: i32,
    pub short_break_minutes: i32,
    pub long_break_minutes: i32,
    pub sessions_until_long_break: i32,
}

impl Default for PomodoroPreset {
    fn default() -> Self {
        Self {
            label: "Classic".to_string(),
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_until_long_break: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

impl SessionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SessionSegmentKind {
    Focus,
    Break,
    Idle,
}

impl SessionSegmentKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::Break => "break",
            Self::Idle => "idle",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

impl ThemePreference {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub planned_focus_minutes: i32,
    pub actual_focus_seconds: i64,
    pub break_seconds: i64,
    pub status: SessionStatus,
    pub preset_label: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SessionSegment {
    pub id: i64,
    pub session_id: i64,
    pub tracked_app_id: Option<i64>,
    pub kind: SessionSegmentKind,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub duration_seconds: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrackedApp {
    pub id: i64,
    pub name: String,
    pub executable: String,
    pub color_hex: Option<String>,
    pub is_excluded: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrackedWindowEvent {
    pub id: i64,
    pub session_id: Option<i64>,
    pub tracked_app_id: Option<i64>,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DailyStat {
    pub date: NaiveDate,
    pub focus_seconds: i64,
    pub break_seconds: i64,
    pub completed_sessions: i32,
    pub interrupted_sessions: i32,
    pub top_app_id: Option<i64>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Achievement {
    pub id: i64,
    pub slug: String,
    pub title: String,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreference {
    pub focus_minutes: i32,
    pub short_break_minutes: i32,
    pub long_break_minutes: i32,
    pub sessions_until_long_break: i32,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
    pub tracking_enabled: bool,
    pub notifications_enabled: bool,
    pub theme: ThemePreference,
    pub updated_at: DateTime<Utc>,
}

impl Default for UserPreference {
    fn default() -> Self {
        Self {
            focus_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            sessions_until_long_break: 4,
            auto_start_breaks: false,
            auto_start_focus: false,
            tracking_enabled: true,
            notifications_enabled: true,
            theme: ThemePreference::System,
            updated_at: Utc::now(),
        }
    }
}
