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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackingCategory {
    Development,
    Browser,
    Communication,
    Writing,
    Design,
    Meeting,
    Research,
    Utilities,
    Unknown,
}

impl TrackingCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Development => "development",
            Self::Browser => "browser",
            Self::Communication => "communication",
            Self::Writing => "writing",
            Self::Design => "design",
            Self::Meeting => "meeting",
            Self::Research => "research",
            Self::Utilities => "utilities",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrackingExclusionKind {
    Executable,
    WindowTitle,
    Category,
}

impl TrackingExclusionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Executable => "executable",
            Self::WindowTitle => "window_title",
            Self::Category => "category",
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
    pub category: TrackingCategory,
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
    pub app_name: Option<String>,
    pub executable: Option<String>,
    pub category: Option<TrackingCategory>,
    pub window_title: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TrackingExclusionRule {
    pub id: i64,
    pub kind: TrackingExclusionKind,
    pub pattern: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
pub struct Streak {
    pub current_days: usize,
    pub best_days: usize,
    pub today_completed: bool,
    pub last_active_date: Option<NaiveDate>,
    pub next_milestone_days: usize,
    pub is_at_risk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WeeklyGoalProgress {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub focus_goal_minutes: i32,
    pub completed_sessions_goal: i32,
    pub focus_minutes_completed: i64,
    pub completed_sessions: usize,
    pub focus_completion_ratio: f64,
    pub sessions_completion_ratio: f64,
    pub completed_goal_count: usize,
    pub is_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressBadge {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub progress_label: String,
    pub progress_ratio: f64,
    pub is_unlocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AchievementProgress {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub progress_current: i64,
    pub progress_target: i64,
    pub progress_ratio: f64,
    pub unlocked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GamificationOverview {
    pub streak: Streak,
    pub weekly_goal: WeeklyGoalProgress,
    pub badges: Vec<ProgressBadge>,
    pub achievements: Vec<AchievementProgress>,
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
    pub tracking_permission_granted: bool,
    pub tracking_onboarding_completed: bool,
    pub notifications_enabled: bool,
    pub sound_enabled: bool,
    pub weekly_focus_goal_minutes: i32,
    pub weekly_completed_sessions_goal: i32,
    pub launch_on_startup: bool,
    pub tray_enabled: bool,
    pub close_to_tray: bool,
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
            tracking_permission_granted: false,
            tracking_onboarding_completed: false,
            notifications_enabled: true,
            sound_enabled: false,
            weekly_focus_goal_minutes: 240,
            weekly_completed_sessions_goal: 5,
            launch_on_startup: false,
            tray_enabled: true,
            close_to_tray: true,
            theme: ThemePreference::System,
            updated_at: Utc::now(),
        }
    }
}
