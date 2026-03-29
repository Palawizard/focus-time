mod models;

pub use models::{
    Achievement, DailyStat, PomodoroPreset, Session, SessionSegment, SessionSegmentKind,
    SessionStatus, ThemePreference, TrackedApp, TrackedWindowEvent, UserPreference,
};

pub const fn crate_name() -> &'static str {
    "focus-domain"
}

#[cfg(test)]
mod tests {
    use super::{PomodoroPreset, SessionStatus, ThemePreference, UserPreference};

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
}
