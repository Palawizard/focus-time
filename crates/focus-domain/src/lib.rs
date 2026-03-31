mod models;
mod pomodoro;

pub use models::{
    Achievement, DailyStat, PomodoroPreset, Session, SessionSegment, SessionSegmentKind,
    SessionStatus, ThemePreference, TrackedApp, TrackedWindowEvent, TrackingCategory,
    TrackingExclusionKind, TrackingExclusionRule, UserPreference,
};
pub use pomodoro::{
    recommended_presets, PomodoroControlState, PomodoroPhase, PomodoroSessionOutcome,
    PomodoroSnapshot, PomodoroTransition, PomodoroTransitionKind,
};

pub const fn crate_name() -> &'static str {
    "focus-domain"
}

#[cfg(test)]
mod tests {
    use super::{
        recommended_presets, PomodoroControlState, PomodoroPhase, PomodoroPreset,
        PomodoroSessionOutcome, PomodoroSnapshot, PomodoroTransitionKind, SessionStatus,
        ThemePreference, UserPreference,
    };
    use chrono::Utc;

    #[test]
    fn exposes_default_preferences() {
        let preferences = UserPreference::default();

        assert_eq!(preferences.focus_minutes, 25);
        assert_eq!(preferences.theme, ThemePreference::System);
    }

    #[test]
    fn builds_a_default_preset() {
        let preset = PomodoroPreset::default();

        assert_eq!(preset.focus_minutes, 25);
        assert_eq!(preset.long_break_minutes, 15);
    }

    #[test]
    fn session_status_values_are_stable() {
        assert_eq!(SessionStatus::Completed.as_str(), "completed");
    }

    #[test]
    fn exposes_recommended_presets() {
        let presets = recommended_presets();

        assert_eq!(presets.len(), 3);
        assert_eq!(presets[0].label, "Classic");
        assert_eq!(presets[1].focus_minutes, 45);
    }

    #[test]
    fn builds_an_idle_pomodoro_snapshot() {
        let snapshot = PomodoroSnapshot::idle(PomodoroPreset::default());

        assert_eq!(snapshot.control_state, PomodoroControlState::Idle);
        assert_eq!(snapshot.phase, None);
        assert_eq!(snapshot.outcome, None);
    }

    #[test]
    fn exposes_transition_kind_labels() {
        assert_eq!(
            PomodoroTransitionKind::FocusCompleted.as_str(),
            "focus_completed"
        );
        assert_eq!(PomodoroSessionOutcome::Interrupted.as_str(), "interrupted");
        assert_eq!(PomodoroPhase::LongBreak.as_str(), "long_break");
    }

    #[test]
    fn snapshot_helpers_match_control_state() {
        let mut snapshot = PomodoroSnapshot::idle(PomodoroPreset::default());
        snapshot.control_state = PomodoroControlState::Running;
        snapshot.phase = Some(PomodoroPhase::Focus);
        snapshot.session_started_at = Some(Utc::now());
        snapshot.can_pause = true;
        snapshot.can_stop = true;

        assert!(snapshot.is_running());
        assert!(snapshot.can_pause);
        assert!(snapshot.can_stop);
    }
}
