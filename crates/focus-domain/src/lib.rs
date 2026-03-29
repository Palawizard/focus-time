use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub const fn crate_name() -> &'static str {
    "focus-domain"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionBlueprint {
    pub started_at: DateTime<Utc>,
    pub planned_minutes: u16,
}

impl SessionBlueprint {
    pub fn new(started_at: DateTime<Utc>, planned_minutes: u16) -> Self {
        Self {
            started_at,
            planned_minutes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SessionBlueprint;
    use chrono::Utc;

    #[test]
    fn creates_a_session_blueprint() {
        let blueprint = SessionBlueprint::new(Utc::now(), 25);

        assert_eq!(blueprint.planned_minutes, 25);
    }
}
