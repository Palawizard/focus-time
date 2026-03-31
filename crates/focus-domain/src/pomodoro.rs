use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::PomodoroPreset;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroPhase {
    Focus,
    ShortBreak,
    LongBreak,
}

impl PomodoroPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Focus => "focus",
            Self::ShortBreak => "short_break",
            Self::LongBreak => "long_break",
        }
    }

    pub const fn is_break(self) -> bool {
        matches!(self, Self::ShortBreak | Self::LongBreak)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroControlState {
    Idle,
    Running,
    Paused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroSessionOutcome {
    Completed,
    Interrupted,
    SkippedBreak,
}

impl PomodoroSessionOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Interrupted => "interrupted",
            Self::SkippedBreak => "skipped_break",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PomodoroTransitionKind {
    SessionStarted,
    Paused,
    Resumed,
    FocusCompleted,
    BreakStarted,
    BreakCompleted,
    SessionStopped,
    BreakSkipped,
    NextFocusStarted,
}

impl PomodoroTransitionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SessionStarted => "session_started",
            Self::Paused => "paused",
            Self::Resumed => "resumed",
            Self::FocusCompleted => "focus_completed",
            Self::BreakStarted => "break_started",
            Self::BreakCompleted => "break_completed",
            Self::SessionStopped => "session_stopped",
            Self::BreakSkipped => "break_skipped",
            Self::NextFocusStarted => "next_focus_started",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroSnapshot {
    pub control_state: PomodoroControlState,
    pub phase: Option<PomodoroPhase>,
    pub preset: PomodoroPreset,
    pub session_started_at: Option<DateTime<Utc>>,
    pub phase_started_at: Option<DateTime<Utc>>,
    pub phase_ends_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub remaining_seconds: i64,
    pub phase_total_seconds: i64,
    pub phase_elapsed_seconds: i64,
    pub focus_seconds_elapsed: i64,
    pub break_seconds_elapsed: i64,
    pub completed_focus_blocks: u32,
    pub completed_breaks: u32,
    pub auto_start_breaks: bool,
    pub auto_start_focus: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub can_stop: bool,
    pub can_skip_break: bool,
    pub session_id: Option<i64>,
    pub outcome: Option<PomodoroSessionOutcome>,
}

impl PomodoroSnapshot {
    pub fn idle(preset: PomodoroPreset) -> Self {
        Self {
            control_state: PomodoroControlState::Idle,
            phase: None,
            preset,
            session_started_at: None,
            phase_started_at: None,
            phase_ends_at: None,
            paused_at: None,
            remaining_seconds: 0,
            phase_total_seconds: 0,
            phase_elapsed_seconds: 0,
            focus_seconds_elapsed: 0,
            break_seconds_elapsed: 0,
            completed_focus_blocks: 0,
            completed_breaks: 0,
            auto_start_breaks: false,
            auto_start_focus: false,
            can_pause: false,
            can_resume: false,
            can_stop: false,
            can_skip_break: false,
            session_id: None,
            outcome: None,
        }
    }

    pub const fn is_running(&self) -> bool {
        matches!(self.control_state, PomodoroControlState::Running)
    }

    pub const fn is_paused(&self) -> bool {
        matches!(self.control_state, PomodoroControlState::Paused)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PomodoroTransition {
    pub id: u64,
    pub kind: PomodoroTransitionKind,
    pub title: String,
    pub body: String,
    pub state: PomodoroSnapshot,
}

pub fn recommended_presets() -> Vec<PomodoroPreset> {
    vec![
        PomodoroPreset::default(),
        PomodoroPreset {
            label: "Deep Work".to_string(),
            focus_minutes: 45,
            short_break_minutes: 10,
            long_break_minutes: 20,
            sessions_until_long_break: 3,
        },
        PomodoroPreset {
            label: "Sprint".to_string(),
            focus_minutes: 60,
            short_break_minutes: 10,
            long_break_minutes: 20,
            sessions_until_long_break: 2,
        },
    ]
}
